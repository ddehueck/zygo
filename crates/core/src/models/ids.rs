use serde::{Deserialize, Serialize};

use crate::models::DomainError;
use crate::models::types::NonEmptyString;

macro_rules! define_value {
    ($name:ident, $label:literal) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(NonEmptyString);

        impl TryFrom<String> for $name {
            type Error = DomainError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                Ok(Self(NonEmptyString::new(value, $label)?))
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.0.as_ref()
            }
        }

        impl From<$name> for String {
            fn from(id: $name) -> String {
                id.0.into_inner()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0.as_ref())
            }
        }
    };
}

define_value!(ChannelId, "channel_id");
define_value!(WorkflowId, "workflow_id");
define_value!(WorkflowVersionId, "workflow_version_id");
define_value!(JobId, "job_id");
define_value!(JobRunId, "job_run_id");
define_value!(RunId, "run_id");
define_value!(EventId, "event_id");

define_value!(WorkflowName, "workflow_name");
define_value!(ChannelName, "channel_name");
define_value!(JobName, "job_name");

define_value!(ContentHash, "content_hash");

define_value!(PythonFunctionName, "python_function_name");
