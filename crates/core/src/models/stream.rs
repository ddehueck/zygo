//! Each stream is scoped to a specific workflow run.
//! - The stream is a sequence of events and commands.
//! - The engine operates on the stream in sequence order.
//! - Clients write to the stream by appending events.
//! - The engine writes commands and events to the stream by appending them.
use serde::{Deserialize, Serialize};

use crate::models::{Command, Event, SequenceId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamRecord {
    pub id: SequenceId,
    pub item: StreamItem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamItem {
    Event(Event),
    Command(Command),
}

