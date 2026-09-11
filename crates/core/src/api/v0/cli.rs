use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::api::error::{self, Result};
use crate::api::v0::interface::{
    DataReference, RunCommandArgs, STDOUT_IPC_PREFIX, StdoutIPCMessage, WorkflowMetadata,
    ZYGO_PKG_INTERNAL_CLI_MODULE,
};
use crate::models::{
    self, Channel, ChannelId, ChannelItemInsertedData, ContentHash, DataReferenceInsertedData,
    Entrypoint, EventKind, FileExtension, Job, JobId, TagInsertedData, WorkflowId, WorkflowSchema,
};

type PythonExecPath = String;
type Cwd = String;
type PythonTarget = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonCli {
    python: PythonExecPath,
    cwd: Cwd,
    target: PythonTarget,
}

impl PythonCli {
    pub fn cwd(&self) -> &str {
        &self.cwd
    }

    pub async fn from_env(target: &str) -> Result<Self> {
        // 1. Find the current python executable in the current working directory
        // Start with `uv python` for now.
        let python = Command::new("uv")
            .args(["python", "find"])
            .output()
            .await
            .map_err(|error| error::Error::UvNotFound(error.to_string()))?;

        if !python.status.success() {
            return Err(error::Error::PythonNotFound);
        }

        let python_path = String::from_utf8_lossy(&python.stdout).trim().to_owned();
        if python_path.is_empty() {
            return Err(error::Error::PythonNotFound);
        }

        let cwd = std::env::current_dir()
            .map_err(|error| error::Error::other(error.to_string()))?
            .to_string_lossy()
            .into_owned();

        // 2. Ensure that the zygo package is in the executable's environment
        let package = Command::new(&python_path)
            .args(["-c", "import zygo"])
            .status()
            .await
            .map_err(|error| error::Error::other(error.to_string()))?;
        if !package.success() {
            return Err(error::Error::ZygoPackageNotFound(python_path));
        }

        Ok(Self {
            python: python_path,
            cwd,
            target: target.to_owned(),
        })
    }

    pub fn build_run_job_command(&self, args: RunCommandArgs) -> Command {
        let mut command = Command::new(self.python.clone());
        command
            // Keep logs and stdout IPC flowing through the shared pipe promptly.
            // Python otherwise block-buffers output when stdout is not a terminal.
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

    pub async fn run_metadata_command(&self) -> Result<WorkflowMetadata> {
        let mut command = Command::new(self.python.clone());
        command.current_dir(&self.cwd).args(vec![
            "-m".into(),
            ZYGO_PKG_INTERNAL_CLI_MODULE.into(),
            "metadata".into(),
            self.target.clone(),
        ]);

        let output = command
            .output()
            .await
            .map_err(|error| error::Error::other(error.to_string()))?;

        if !output.status.success() {
            return Err(error::Error::other("metadata command failed"));
        }

        let response = String::from_utf8_lossy(&output.stdout);
        Self::parse_metadata_response(&response)
    }

    fn parse_metadata_response(response: &str) -> Result<WorkflowMetadata> {
        let payload = response
            .lines()
            .find_map(|line| line.strip_prefix(STDOUT_IPC_PREFIX))
            .ok_or_else(|| crate::api::error::Error::other("metadata IPC response not found"))?;
        let metadata: WorkflowMetadata = serde_json::from_str(payload)?;
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
                value,
                data_reference,
            } => {
                Self::TagInserted(TagInsertedData {
                    value,
                    // todo: this conversion is funky
                    data_reference: data_reference.map(models::DataReference::from),
                })
            }
        })
    }
}
