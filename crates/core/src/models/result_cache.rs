use serde::{Deserialize, Serialize};

use crate::store::StoreKey;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultCacheItem {
    pub event_keys: Vec<StoreKey>,
}
