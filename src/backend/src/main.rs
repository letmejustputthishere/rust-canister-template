use backend::dashboard::DashboardTemplate;
use backend::lifecycle::Arg;
use backend::logs::INFO;
use backend::state::audit::replay_events;
use backend::state::{mutate_state, State};
use backend::storage::record_event;
use backend::{
    metrics::encode_metrics,
    state::{self, initialize_state, read_state},
};
use ic_canister_log::log;

#[ic_cdk::update]
fn greet(name: String) -> String {
    record_event(name.clone());
    // insert the name into the greeted_names_count map
    mutate_state(|s| {
        s.greeted_names_count
            .entry(name.clone())
            .and_modify(|count| *count += 1)
            .or_insert(1);
    });
    format!("{}, {}!", read_state(|s| s.greeting.clone()), name)
}

#[ic_cdk::query]
fn total_greeted_names_count() -> u64 {
    // this helps avoding expensive query calls to be executed in replicated mode
    if ic_cdk::api::in_replicated_execution() {
        ic_cdk::trap("update call rejected");
    }
    read_state(|s| s.greeted_names_count.len() as u64)
}

#[ic_cdk::query]
fn greeted_name_count(name: String) -> u64 {
    // this helps avoding expensive query calls to be executed in replicated mode
    if ic_cdk::api::in_replicated_execution() {
        ic_cdk::trap("update call rejected");
    }
    read_state(|s| {
        s.greeted_names_count
            .get(&name)
            .copied()
            .unwrap_or_default()
    })
}

#[ic_cdk::init]
fn init(arg: Arg) {
    match arg {
        Arg::InitArg(init_arg) => {
            log!(INFO, "[init]: initialized minter with arg: {:?}", init_arg);
            initialize_state(
                state::State::try_from(init_arg.clone())
                    .expect("BUG: failed to initialize canister"),
            );
        }
        Arg::UpgradeArg(_) => {
            ic_cdk::trap("cannot init canister state with upgrade args");
        }
    }
}

#[ic_cdk::post_upgrade]
fn post_upgrade(arg: Arg) {
    match arg {
        Arg::InitArg(_) => {
            ic_cdk::trap("cannot upgrade canister state with init args");
        }
        Arg::UpgradeArg(upgrade_arg) => {
            initialize_state(State {
                greeted_names_count: {
                    #[cfg(feature = "canbench-rs")]
                    let _p = canbench_rs::bench_scope("replay_events");
                    replay_events()
                },
                ..State::try_from(upgrade_arg.clone()).expect("BUG: failed to initialize canister")
            });
            log!(
                INFO,
                "[upgrade]: upgraded canister with arg: {:?}",
                upgrade_arg
            );
        }
    }
}

#[ic_cdk::query(hidden = true)]
fn http_request(req: backend::http_types::HttpRequest) -> backend::http_types::HttpResponse {
    use backend::http_types::HttpResponseBuilder;

    // this helps avoding expensive query calls to be executed in replicated mode
    if ic_cdk::api::in_replicated_execution() {
        ic_cdk::trap("update call rejected");
    }

    if req.path() == "/metrics" {
        let mut writer =
            ic_metrics_encoder::MetricsEncoder::new(vec![], ic_cdk::api::time() as i64 / 1_000_000);

        match encode_metrics(&mut writer) {
            Ok(()) => HttpResponseBuilder::ok()
                .header("Content-Type", "text/plain; version=0.0.4")
                .with_body_and_content_length(writer.into_inner())
                .build(),
            Err(err) => {
                HttpResponseBuilder::server_error(format!("Failed to encode metrics: {}", err))
                    .build()
            }
        }
    } else if req.path() == "/dashboard" {
        use askama::Template;

        let dashboard = read_state(DashboardTemplate::from_state);
        HttpResponseBuilder::ok()
            .header("Content-Type", "text/html; charset=utf-8")
            .with_body_and_content_length(dashboard.render().unwrap())
            .build()
    } else if req.path() == "/logs" {
        use backend::logs::{Log, Priority, Sort};
        use std::str::FromStr;

        let max_skip_timestamp = match req.raw_query_param("time") {
            Some(arg) => match u64::from_str(arg) {
                Ok(value) => value,
                Err(_) => {
                    return HttpResponseBuilder::bad_request()
                        .with_body_and_content_length("failed to parse the 'time' parameter")
                        .build();
                }
            },
            None => 0,
        };

        let mut log: Log = Default::default();

        match req.raw_query_param("priority") {
            Some(priority_str) => match Priority::from_str(priority_str) {
                Ok(priority) => match priority {
                    Priority::Info => log.push_logs(Priority::Info),
                    Priority::Debug => log.push_logs(Priority::Debug),
                },
                Err(_) => log.push_all(),
            },
            None => log.push_all(),
        }

        log.entries
            .retain(|entry| entry.timestamp >= max_skip_timestamp);

        fn ordering_from_query_params(sort: Option<&str>, max_skip_timestamp: u64) -> Sort {
            match sort {
                Some(ord_str) => match Sort::from_str(ord_str) {
                    Ok(order) => order,
                    Err(_) => {
                        if max_skip_timestamp == 0 {
                            Sort::Ascending
                        } else {
                            Sort::Descending
                        }
                    }
                },
                None => {
                    if max_skip_timestamp == 0 {
                        Sort::Ascending
                    } else {
                        Sort::Descending
                    }
                }
            }
        }

        log.sort_logs(ordering_from_query_params(
            req.raw_query_param("sort"),
            max_skip_timestamp,
        ));

        const MAX_BODY_SIZE: usize = 2_000_000;
        HttpResponseBuilder::ok()
            .header("Content-Type", "application/json; charset=utf-8")
            .with_body_and_content_length(log.serialize_logs(MAX_BODY_SIZE))
            .build()
    } else {
        HttpResponseBuilder::not_found().build()
    }
}

fn main() {}

// Enable Candid export
ic_cdk::export_candid!();

#[cfg(feature = "canbench-rs")]
mod benches {
    use super::*;
    use backend::lifecycle::UpgradeArg;
    use canbench_rs::bench;

    // Benchmarks inserting 1 million users into the state.
    #[bench]
    fn insert_greetings() {
        mutate_state(|s| {
            for i in 0..1_000_000 {
                s.greeted_names_count.insert(i.to_string(), 1);
            }
        })
    }

    #[bench]
    fn insert_events() {
        for i in 0..1_000_000 {
            record_event(i.to_string());
        }
    }

    // Benchmarks removing 1 million users from the state.
    #[bench(raw)]
    fn remove_greetings() -> canbench_rs::BenchResult {
        insert_greetings();

        // Only benchmark removing users. Inserting users isn't
        // included in the results of our benchmark.
        canbench_rs::bench_fn(|| {
            mutate_state(|s| {
                for i in 0..1_000_000 {
                    s.greeted_names_count.remove(&i.to_string());
                }
            })
        })
    }

    #[bench(raw)]
    fn post_upgrade_bench() -> canbench_rs::BenchResult {
        // this shouldnt affect the benchmark results as
        // post_upgrade is only relying on the event log
        insert_events();

        // Only benchmark the pre_upgrade. Inserting users isn't
        // included in the results of our benchmark.
        canbench_rs::bench_fn(|| {
            post_upgrade(Arg::UpgradeArg(UpgradeArg {
                greeting: "hoi".to_string(),
            }))
        })
    }
}
