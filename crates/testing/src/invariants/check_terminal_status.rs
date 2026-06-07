use super::{Invariant, InvariantOutcome};

pub struct CheckTerminalStatus {
    hit_timeout: bool,
}

impl CheckTerminalStatus {
    pub fn new(hit_timeout: bool) -> Self {
        Self { hit_timeout }
    }
}

impl Invariant for CheckTerminalStatus {
    fn name(&self) -> &str {
        "check_terminal_status"
    }

    fn check(&self) -> InvariantOutcome {
        if self.hit_timeout {
            InvariantOutcome::Failed(vec!["Hit timeout".to_string()])
        } else {
            InvariantOutcome::Passed
        }
    }
}
