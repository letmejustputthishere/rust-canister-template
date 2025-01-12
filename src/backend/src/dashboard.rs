use askama::Template;

use crate::state::State;

#[derive(Template)]
#[template(path = "dashboard.html")]
#[derive(Clone)]
pub struct DashboardTemplate {
    pub greeting: String,
}

impl DashboardTemplate {
    pub fn from_state(state: &State) -> Self {
        DashboardTemplate {
            greeting: state.greeting.clone(),
        }
    }
}
