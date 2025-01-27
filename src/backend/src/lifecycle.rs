use candid::CandidType;
use serde::Deserialize;

use crate::state::{InvalidStateError, State};

#[derive(Clone, Eq, PartialEq, Debug, CandidType, Deserialize)]
pub struct InitArg {
    pub greeting: String,
}

#[derive(Clone, Eq, PartialEq, Debug, CandidType, Deserialize)]
pub struct UpgradeArg {
    pub greeting: String,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum Arg {
    InitArg(InitArg),
    UpgradeArg(UpgradeArg),
}

impl TryFrom<InitArg> for State {
    type Error = InvalidStateError;
    fn try_from(InitArg { greeting }: InitArg) -> Result<Self, Self::Error> {
        let state = Self {
            greeting,
            greeted_names_count: Default::default(),
        };
        state.validate_config()?;
        Ok(state)
    }
}

impl TryFrom<UpgradeArg> for State {
    type Error = InvalidStateError;
    fn try_from(UpgradeArg { greeting }: UpgradeArg) -> Result<Self, Self::Error> {
        let state = Self {
            greeting,
            greeted_names_count: Default::default(),
        };
        state.validate_config()?;
        Ok(state)
    }
}
