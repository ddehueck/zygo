use serde::{Deserialize, Serialize};

use super::ids::{WorkflowId, WorkflowName};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: WorkflowId,
    pub name: WorkflowName,
}
