use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

///! A sequence id is a monotically increasing positive integer that is lexicographically sortable.
///! - It is used as the primary key for a stream.
///! - It is implemented as a fixed-width, zero-padded integer key

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SequenceId(u64);

const DISPLAY_WIDTH: usize = 20;

impl SequenceId {
    pub fn new(v: u64) -> Self {
        Self(v)
    }

    pub fn get(self) -> u64 {
        self.0
    }

    pub fn next(self) -> Self {
        Self(self.0 + 1)
    }

    pub fn checked_next(self) -> Option<Self> {
        self.0.checked_add(1).map(Self)
    }

    pub fn to_bytes(self) -> [u8; std::mem::size_of::<u64>()] {
        self.0.to_be_bytes()
    }
}

impl fmt::Display for SequenceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:0width$}", self.0, width = DISPLAY_WIDTH)
    }
}

impl std::str::FromStr for SequenceId {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

impl TryFrom<String> for SequenceId {
    type Error = std::num::ParseIntError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}
