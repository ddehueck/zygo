use serde::{Deserialize, Serialize};

/// A monotonically increasing stream key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SequenceId(u64);

impl SequenceId {
    pub fn new(v: u64) -> Self {
        Self(v)
    }

    pub fn get(self) -> u64 {
        self.0
    }

    pub fn increment(self) -> Self {
        Self(self.0 + 1)
    }
}
