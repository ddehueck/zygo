use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

use serde_json::Value;

use crate::{dependencies::StorageProvider, store::StoreKey};

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
    async fn put(&self, entries: &[(StoreKey, Value)]) -> Result<(), anyhow::Error> {
        let mut stored_entries = self
            .entries
            .write()
            .map_err(|err| anyhow::anyhow!("memory store write lock poisoned: {err}"))?;

        for (key, value) in entries {
            stored_entries.insert(key.as_str().to_owned(), value.clone());
        }

        Ok(())
    }

    async fn get(&self, key: &StoreKey) -> Result<Option<Value>, anyhow::Error> {
        let entries = self
            .entries
            .read()
            .map_err(|err| anyhow::anyhow!("memory store read lock poisoned: {err}"))?;

        Ok(entries.get(key.as_str()).cloned())
    }

    async fn get_many(&self, keys: &[StoreKey]) -> Result<Vec<Option<Value>>, anyhow::Error> {
        let entries = self
            .entries
            .read()
            .map_err(|err| anyhow::anyhow!("memory store read lock poisoned: {err}"))?;

        Ok(keys
            .iter()
            .map(|key| entries.get(key.as_str()).cloned())
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{MemoryStore, StorageProvider, StoreKey};

    #[tokio::test]
    async fn stores_and_fetches_json_values() {
        let store = MemoryStore::new();
        let one = json!({ "name": "one" });
        let two = json!([2, "two"]);
        let three = json!(3);

        let a = StoreKey::from("a");
        let b = StoreKey::from("b");
        let c = StoreKey::from("c");
        store
            .put(&[
                (a.clone(), one.clone()),
                (b.clone(), two),
                (c.clone(), three.clone()),
            ])
            .await
            .unwrap();

        assert_eq!(store.get(&a).await.unwrap(), Some(one));
        assert_eq!(
            store
                .get_many(&[a, StoreKey::from("missing"), c])
                .await
                .unwrap(),
            vec![Some(json!({ "name": "one" })), None, Some(three)]
        );
    }

    #[tokio::test]
    async fn clones_share_the_same_entries() {
        let store = MemoryStore::new();
        let cloned_store = store.clone();
        let value = json!({ "shared": true });

        let key = StoreKey::from("key");
        store.put(&[(key.clone(), value.clone())]).await.unwrap();

        assert_eq!(cloned_store.get(&key).await.unwrap(), Some(value));
    }
}
