use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

use serde_json::Value;

use super::StorageProvider;

#[derive(Debug, Clone, Default)]
pub struct MemoryStore {
    entries: Arc<RwLock<BTreeMap<String, Value>>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl StorageProvider for MemoryStore {
    async fn put(&self, entries: &[(&str, &Value)]) -> Result<(), anyhow::Error> {
        let mut stored_entries = self
            .entries
            .write()
            .map_err(|err| anyhow::anyhow!("memory store write lock poisoned: {err}"))?;

        for (key, value) in entries {
            stored_entries.insert((*key).to_owned(), (*value).clone());
        }

        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Option<Value>, anyhow::Error> {
        let entries = self
            .entries
            .read()
            .map_err(|err| anyhow::anyhow!("memory store read lock poisoned: {err}"))?;

        Ok(entries.get(key).cloned())
    }

    async fn get_many(&self, keys: &[&str]) -> Result<Vec<Option<Value>>, anyhow::Error> {
        let entries = self
            .entries
            .read()
            .map_err(|err| anyhow::anyhow!("memory store read lock poisoned: {err}"))?;

        Ok(keys.iter().map(|key| entries.get(*key).cloned()).collect())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::MemoryStore;
    use crate::store::StorageProvider;

    #[tokio::test]
    async fn stores_and_fetches_json_values() {
        let store = MemoryStore::new();
        let one = json!({ "name": "one" });
        let two = json!([2, "two"]);
        let three = json!(3);

        store
            .put(&[("a", &one), ("b", &two), ("c", &three)])
            .await
            .unwrap();

        assert_eq!(store.get("a").await.unwrap(), Some(one));
        assert_eq!(
            store.get_many(&["a", "missing", "c"]).await.unwrap(),
            vec![Some(json!({ "name": "one" })), None, Some(three)]
        );
    }

    #[tokio::test]
    async fn clones_share_the_same_entries() {
        let store = MemoryStore::new();
        let cloned_store = store.clone();
        let value = json!({ "shared": true });

        store.put(&[("key", &value)]).await.unwrap();

        assert_eq!(cloned_store.get("key").await.unwrap(), Some(value));
    }
}
