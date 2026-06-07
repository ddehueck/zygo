use tracing::{error, info};

use super::{Invariant, InvariantOutcome};

/// Runs prepared invariants and logs whether each one passed or failed.
#[derive(Debug, Default, Clone, Copy)]
pub struct InvariantRunner;

impl InvariantRunner {
    pub fn run(&self, invariant: &dyn Invariant) -> bool {
        let name = invariant.name();

        match invariant.check() {
            InvariantOutcome::Passed => {
                info!(invariant = name, "invariant passed");
                true
            }
            InvariantOutcome::Failed(violations) => {
                for violation in violations {
                    error!(invariant = name, "{violation}");
                }
                false
            }
        }
    }
}
