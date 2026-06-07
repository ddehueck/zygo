use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataReference {
    pub uri: String,
    pub etag: String,
    pub content_type: Option<String>,
    pub size_bytes: Option<u64>,
}

impl DataReference {
    pub fn new(uri: String, etag: String) -> Self {
        Self {
            uri,
            etag,
            content_type: None,
            size_bytes: None,
        }
    }
}
