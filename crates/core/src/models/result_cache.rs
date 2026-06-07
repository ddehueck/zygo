use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultCacheItem {
    pub event_keys: Vec<String>,
}
