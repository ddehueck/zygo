use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::DomainError;

macro_rules! define_value {
    ($name:ident, $label:literal) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl TryFrom<String> for $name {
            type Error = DomainError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                if value.trim().is_empty() {
                    Err(DomainError::empty($label))
                } else {
                    Ok(Self(value))
                }
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }

        impl From<$name> for String {
            fn from(id: $name) -> String {
                id.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

define_value!(WorkflowId, "workflow_id");
define_value!(WorkflowRunId, "workflow_run_id");
define_value!(ChannelId, "channel_id");
define_value!(JobId, "job_id");
define_value!(JobRunId, "job_run_id");
define_value!(EventId, "event_id");
define_value!(ContentHash, "content_hash");
define_value!(PythonFunctionName, "python_function_name");

impl EventId {
    pub fn new() -> Self {
        Self::try_from(Uuid::now_v7().to_string()).expect("generated UUID must be a valid event ID")
    }
}

impl WorkflowRunId {
    /// Creates a unique workflow execution attempt.
    ///
    /// Workflow run IDs must not double as result-cache keys: restarting the same
    /// workflow and inputs should create a fresh stream, while deterministic job
    /// run IDs independently reuse results from jobs that completed successfully.
    pub fn new() -> Self {
        Self::try_from(Uuid::now_v7().to_string())
            .expect("generated UUID must be a valid workflow run ID")
    }
}

#[cfg(test)]
mod tests {
    use super::WorkflowRunId;

    #[test]
    fn workflow_execution_attempts_have_unique_ids() {
        assert_ne!(WorkflowRunId::new(), WorkflowRunId::new());
    }
}
