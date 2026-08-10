use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::{
    ipc::{
        error::Result,
        v0::interface::{
            DataReference, RunCommandArgs, STDOUT_IPC_PREFIX, StdoutIPCMessage, WorkflowMetadata,
            ZYGO_PKG_INTERNAL_CLI_MODULE,
        },
    },
    models::{self, ChannelItemInsertedData, DataReferenceInsertedData, EventKind},
};

/// This struct serves as the interface for interacting with the v0 python cli
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonCli {
    python_exec: String,
    cwd: String,
    target: String,
}

impl PythonCli {
    pub fn new(python_exec_path: String, cwd: String, target: String) -> Self {
        return Self {
            python_exec: python_exec_path,
            cwd: cwd,
            target: target,
        };
    }

    pub fn run_entrypoint(&self, args: RunCommandArgs) -> Command {
        let mut command = Command::new(self.python_exec.clone());
        command.current_dir(&self.cwd).args(vec![
            "-m".into(),
            ZYGO_PKG_INTERNAL_CLI_MODULE.into(),
            "run".into(),
            self.target.clone(),
            "--args".into(),
            serde_json::to_string(&args).expect("failed to serialze RunCommandArgs"),
        ]);
        command
    }

    pub fn parse_run_stdout(line: &str) -> Result<Option<EventKind>> {
        if let Some(payload) = line.strip_prefix(STDOUT_IPC_PREFIX) {
            let message: StdoutIPCMessage = serde_json::from_str(payload)?;
            return Ok(Some(EventKind::try_from(message)?));
        }
        Ok(None)
    }

    pub fn metadata_entrypoint(&self) -> Command {
        let mut command = Command::new(self.python_exec.clone());
        command.current_dir(&self.cwd).args(vec![
            "-m".into(),
            ZYGO_PKG_INTERNAL_CLI_MODULE.into(),
            "metadata".into(),
            self.target.clone(),
        ]);
        command
    }

    pub fn parse_metadata_response(response: &str) -> Result<WorkflowMetadata> {
        let metadata: WorkflowMetadata = serde_json::from_str(response)?;
        Ok(metadata)
    }
}

impl From<DataReference> for models::DataReference {
    fn from(data_reference: DataReference) -> Self {
        Self {
            uri: data_reference.uri,
            etag: data_reference.etag,
            content_type: data_reference.content_type,
            size_bytes: data_reference.size_bytes,
        }
    }
}

impl TryFrom<StdoutIPCMessage> for EventKind {
    type Error = anyhow::Error;

    fn try_from(message: StdoutIPCMessage) -> std::result::Result<Self, Self::Error> {
        Ok(match message {
            StdoutIPCMessage::DataReferenceCreated { data_reference } => {
                Self::DataReferenceInserted(DataReferenceInsertedData {
                    data_reference: models::DataReference::from(data_reference),
                })
            }
            StdoutIPCMessage::ChannelItemInserted {
                channel_id,
                data_reference,
            } => Self::ChannelItemInserted(ChannelItemInsertedData {
                // TODO: This should be just a from?
                channel_id: models::ChannelId::try_from(channel_id)?,
                data_reference: models::DataReference::from(data_reference),
            }),
        })
    }
}
