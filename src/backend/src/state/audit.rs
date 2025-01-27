use std::collections::HashMap;

use crate::storage::with_event_iter;

pub fn replay_events() -> HashMap<String, u64> {
    let mut greeted_names_count = HashMap::new();
    with_event_iter(|events| {
        for event in events {
            greeted_names_count
                .entry(event)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
    });
    greeted_names_count
}
