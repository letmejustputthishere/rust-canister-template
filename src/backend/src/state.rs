use std::cell::RefCell;

thread_local! {
    pub static STATE: RefCell<Option<State>> = RefCell::default();
}

#[derive(Clone, PartialEq, Debug)]
pub struct State {
    pub greeting: String,
}

#[derive(Eq, PartialEq, Debug)]
pub enum InvalidStateError {
    InvalidGreeting(String),
}

impl State {
    pub fn validate_config(&self) -> Result<(), InvalidStateError> {
        if self.greeting.trim().is_empty() {
            return Err(InvalidStateError::InvalidGreeting(
                "greeting cannot be blank".to_string(),
            ));
        }
        Ok(())
    }
}

/// Reads the current state using `f`.
pub fn read_state<R>(f: impl FnOnce(&State) -> R) -> R {
    STATE.with_borrow(|s| f(s.as_ref().expect("BUG: state is not initialized")))
}

/// Mutates (part of) the current state using `f`.
///
/// Panics if there is no state.
pub fn mutate_state<F, R>(f: F) -> R
where
    F: FnOnce(&mut State) -> R,
{
    STATE.with_borrow_mut(|s| f(s.as_mut().expect("BUG: state is not initialized")))
}

/// Sets the current state to `state`.
pub fn initialize_state(state: State) {
    STATE.set(Some(state));
}
