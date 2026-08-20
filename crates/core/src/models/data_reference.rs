use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataReference {
    pub uri: String,
    pub etag: String,
}

impl DataReference {
    pub fn new(uri: String, etag: String) -> Self {
        Self { uri, etag }
    }
}
