use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalEntrypoint {
    pub cwd: String,
    pub exec: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteEntrypoint {
    pub url: String,
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobEntrypoint {
    Local(LocalEntrypoint),
    Remote(RemoteEntrypoint),
}
