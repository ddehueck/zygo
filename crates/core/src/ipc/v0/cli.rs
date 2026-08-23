use std::process::Command as StdCommand;

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
    models::{
        self, Channel, ChannelId, ChannelItemInsertedData, ContentHash, DataReferenceInsertedData,
        Entrypoint, EventKind, FileExtension, Job, JobId, TagInsertedData, WorkflowId,
        WorkflowSchema,
    },
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
        Self {
            python_exec: python_exec_path,
            cwd,
            target,
        }
    }

    pub fn cwd(&self) -> &str {
        &self.cwd
    }

    pub fn run_entrypoint(&self, args: RunCommandArgs) -> Command {
        let mut command = Command::new(self.python_exec.clone());
        // Keep logs and stdout IPC flowing through the shared pipe promptly.
        // Python otherwise block-buffers output when stdout is not a terminal.
        command
            .env("PYTHONUNBUFFERED", "1")
            .current_dir(&self.cwd)
            .args(vec![
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

    pub fn metadata_entrypoint(&self) -> StdCommand {
        let mut command = StdCommand::new(self.python_exec.clone());
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

    /// Builds the runtime schema returned by this entrypoint's metadata command.
    pub fn workflow_schema_from_metadata(
        &self,
        metadata: WorkflowMetadata,
    ) -> Result<WorkflowSchema> {
        let content_hash = ContentHash::try_from(metadata.content_hash)
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        let entrypoint = Entrypoint::Python(self.clone());
        let channels = metadata
            .channels
            .into_iter()
            .map(|channel| {
                Ok(Channel {
                    id: ChannelId::try_from(channel.id)
                        .map_err(|error| anyhow::anyhow!(error.to_string()))?,
                    accepted_file_extensions: channel
                        .accepted_file_extensions
                        .into_iter()
                        .map(FileExtension::from)
                        .collect(),
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let jobs = metadata
            .jobs
            .into_iter()
            .map(|job| {
                Ok(Job {
                    id: JobId::try_from(job.id)
                        .map_err(|error| anyhow::anyhow!(error.to_string()))?,
                    content_hash: ContentHash::try_from(job.content_hash)
                        .map_err(|error| anyhow::anyhow!(error.to_string()))?,
                    input_channel_id: ChannelId::try_from(job.input_channel_id)
                        .map_err(|error| anyhow::anyhow!(error.to_string()))?,
                    output_channel_id: ChannelId::try_from(job.output_channel_id)
                        .map_err(|error| anyhow::anyhow!(error.to_string()))?,
                    entrypoint: entrypoint.clone(),
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(WorkflowSchema {
            id: WorkflowId::try_from(metadata.id)
                .map_err(|error| anyhow::anyhow!(error.to_string()))?,
            entrypoint,
            content_hash,
            input_channel_id: ChannelId::try_from(metadata.input_channel_id)
                .map_err(|error| anyhow::anyhow!(error.to_string()))?,
            output_channel_id: ChannelId::try_from(metadata.output_channel_id)
                .map_err(|error| anyhow::anyhow!(error.to_string()))?,
            jobs,
            channels,
        })
    }
}

impl From<DataReference> for models::DataReference {
    fn from(data_reference: DataReference) -> Self {
        Self {
            uri: data_reference.uri,
            version: data_reference.version,
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
            StdoutIPCMessage::TagInserted {
                name,
                value,
                data_reference,
            } => Self::TagInserted(TagInsertedData {
                name,
                value,
                // todo: this conversion is funky
                data_reference: data_reference.map(models::DataReference::from),
            }),
        })
    }
}
