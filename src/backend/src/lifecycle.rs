use candid::CandidType;
use serde::Deserialize;

use crate::state::{InvalidStateError, State};

#[derive(Clone, Eq, PartialEq, Debug, CandidType, Deserialize)]
pub struct InstallArgs {
    pub greeting: String,
}

impl TryFrom<InstallArgs> for State {
    type Error = InvalidStateError;
    fn try_from(InstallArgs { greeting }: InstallArgs) -> Result<Self, Self::Error> {
        let state = Self { greeting };
        state.validate_config()?;
        Ok(state)
    }
}
