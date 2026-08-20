use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataReference {
    pub uri: String,
    pub version: String,
}

impl DataReference {
    pub fn new(uri: String, version: String) -> Self {
        Self { uri, version }
    }
}
