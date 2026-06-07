use serde::{Deserialize, Serialize};

use super::sequence_id::SequenceId;

/// Next sequence id to assign for stream appends (exclusive tail), persisted at
/// `RunKeySpace::stream_append_cursor`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StreamAppendCursor {
    pub next_append_sequence_id: SequenceId,
}
