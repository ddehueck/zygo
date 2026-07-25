use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{DataReference, DomainError};

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

define_value!(ChannelId, "channel_id");
define_value!(WorkflowRunId, "workflow_run_id");
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

/// Namespace for generating deterministic workflow run IDs.
const WORKFLOW_RUN_NAMESPACE: Uuid = Uuid::from_u128(0x6ba7_b812_9dad_11d1_80b4_00c0_4fd4_30c8);

impl WorkflowRunId {
    pub fn new(
        workflow_schema_content_hash: &ContentHash,
        data_reference: &DataReference,
    ) -> Result<Self, DomainError> {
        let name = format!(
            "{}\0{}\0{}",
            workflow_schema_content_hash.as_ref(),
            data_reference.etag,
            data_reference.uri
        );
        Self::try_from(Uuid::new_v5(&WORKFLOW_RUN_NAMESPACE, name.as_bytes()).to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_accepts_non_empty_string() {
        let id = JobId::try_from("job-id".to_string()).unwrap();

        assert_eq!(id.as_ref(), "job-id");
        assert_eq!(String::from(id), "job-id");
    }

    #[test]
    fn value_rejects_empty_string() {
        let error = JobId::try_from(String::new()).unwrap_err();

        assert_eq!(error.to_string(), "job_id cannot be empty");
    }

    #[test]
    fn value_rejects_whitespace_only_string() {
        let error = JobId::try_from("   ".to_string()).unwrap_err();

        assert_eq!(error.to_string(), "job_id cannot be empty");
    }
}
