use nutype::nutype;
use uuid::Uuid;

#[nutype(
    validate(predicate = |value: &str| !value.trim().is_empty()),
    derive(Debug, Clone, PartialEq, Eq, Hash, AsRef, Display, Into, TryFrom, Serialize, Deserialize),
)]
pub struct WorkflowId(String);

#[nutype(
    validate(predicate = |value: &str| !value.trim().is_empty()),
    derive(Debug, Clone, PartialEq, Eq, Hash, AsRef, Display, Into, TryFrom, Serialize, Deserialize),
)]
pub struct WorkflowRunId(String);

#[nutype(
    validate(predicate = |value: &str| !value.trim().is_empty()),
    derive(Debug, Clone, PartialEq, Eq, Hash, AsRef, Display, Into, TryFrom, Serialize, Deserialize),
)]
pub struct ChannelId(String);

#[nutype(
    validate(predicate = |value: &str| !value.trim().is_empty()),
    derive(Debug, Clone, PartialEq, Eq, Hash, AsRef, Display, Into, TryFrom, Serialize, Deserialize),
)]
pub struct JobId(String);

#[nutype(
    validate(predicate = |value: &str| !value.trim().is_empty()),
    derive(Debug, Clone, PartialEq, Eq, Hash, AsRef, Display, Into, TryFrom, Serialize, Deserialize),
)]
pub struct JobRunId(String);

#[nutype(
    validate(predicate = |value: &str| !value.trim().is_empty()),
    derive(Debug, Clone, PartialEq, Eq, Hash, AsRef, Display, Into, TryFrom, Serialize, Deserialize),
)]
pub struct EventId(String);

#[nutype(
    validate(predicate = |value: &str| !value.trim().is_empty()),
    derive(Debug, Clone, PartialEq, Eq, Hash, AsRef, Display, Into, TryFrom, Serialize, Deserialize),
)]
pub struct DataReferenceUri(String);

#[nutype(
    validate(predicate = |value: &str| !value.trim().is_empty()),
    derive(Debug, Clone, PartialEq, Eq, Hash, AsRef, Display, Into, TryFrom, Serialize, Deserialize),
)]
pub struct ContentHash(String);

#[nutype(
    validate(predicate = |value: &str| !value.trim().is_empty()),
    derive(Debug, Clone, PartialEq, Eq, Hash, AsRef, Display, Into, TryFrom, Serialize, Deserialize),
)]
pub struct PythonFunctionName(String);

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
    use super::{WorkflowId, WorkflowRunId};

    #[test]
    fn workflow_execution_attempts_have_unique_ids() {
        assert_ne!(WorkflowRunId::new(), WorkflowRunId::new());
    }

    #[test]
    fn ids_reject_empty_and_whitespace_only_values() {
        assert!(WorkflowId::try_from(String::new()).is_err());
        assert!(WorkflowId::try_from("   ".to_owned()).is_err());
        let id = WorkflowId::try_from("workflow-1".to_owned()).unwrap();
        assert_eq!(id.as_ref(), "workflow-1");
        assert_eq!(id.to_string(), "workflow-1");
        assert_eq!(String::from(id), "workflow-1");
    }
}
