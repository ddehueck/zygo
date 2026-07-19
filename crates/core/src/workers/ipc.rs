use serde::Deserialize;

use crate::models::{
    ChannelId, ChannelItemInsertedData, DataReference, DataReferenceInsertedData, EventKind,
};

const STDOUT_IPC_PREFIX: &str = "ZYGO_IPC=";

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StdoutIPCMessage {
    DataReferenceCreated {
        data_reference: DataReference,
    },
    ChannelItemInserted {
        channel_id: ChannelId,
        data_reference: DataReference,
    },
}

impl From<StdoutIPCMessage> for EventKind {
    fn from(message: StdoutIPCMessage) -> Self {
        match message {
            StdoutIPCMessage::DataReferenceCreated { data_reference } => {
                Self::DataReferenceInserted(DataReferenceInsertedData { data_reference })
            }
            StdoutIPCMessage::ChannelItemInserted {
                channel_id,
                data_reference,
            } => Self::ChannelItemInserted(ChannelItemInsertedData {
                channel_id,
                data_reference,
            }),
        }
    }
}

pub fn parse_line(line: &str) -> Result<Option<StdoutIPCMessage>, serde_json::Error> {
    if let Some(payload) = line.strip_prefix(STDOUT_IPC_PREFIX) {
        return Ok(Some(serde_json::from_str(payload)?));
    }
    Ok(None)
}
