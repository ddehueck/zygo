use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

use super::StorageProvider;

#[derive(Debug, Clone, Default)]
pub struct MemoryStore {
    entries: Arc<RwLock<BTreeMap<String, Vec<u8>>>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl StorageProvider for MemoryStore {
    fn put(
        &self,
        key: &str,
        value: &[u8],
    ) -> impl Future<Output = Result<(), anyhow::Error>> + Send {
        let key = key.to_owned();
        let value = value.to_vec();
        let entries = Arc::clone(&self.entries);

        async move {
            let mut entries = entries
                .write()
                .map_err(|err| anyhow::anyhow!("memory store write lock poisoned: {err}"))?;
            entries.insert(key, value);
            Ok(())
        }
    }

    fn put_many<'a>(
        &'a self,
        entries: &'a [(&'a str, &'a [u8])],
    ) -> impl Future<Output = Result<(), anyhow::Error>> + Send {
        let batch: Vec<(String, Vec<u8>)> = entries
            .iter()
            .map(|(key, value)| ((*key).to_owned(), (*value).to_vec()))
            .collect();
        let stored_entries = Arc::clone(&self.entries);

        async move {
            let mut stored_entries = stored_entries
                .write()
                .map_err(|err| anyhow::anyhow!("memory store write lock poisoned: {err}"))?;

            for (key, value) in batch {
                stored_entries.insert(key, value);
            }

            Ok(())
        }
    }

    fn get(
        &self,
        key: &str,
    ) -> impl Future<Output = Result<Option<Vec<u8>>, anyhow::Error>> + Send {
        let key = key.to_owned();
        let entries = Arc::clone(&self.entries);

        async move {
            let entries = entries
                .read()
                .map_err(|err| anyhow::anyhow!("memory store read lock poisoned: {err}"))?;
            Ok(entries.get(&key).cloned())
        }
    }

    fn get_many(
        &self,
        keys: &[&str],
    ) -> impl Future<Output = Result<Vec<Option<Vec<u8>>>, anyhow::Error>> + Send {
        let keys: Vec<String> = keys.iter().map(|key| (*key).to_owned()).collect();
        let entries = Arc::clone(&self.entries);

        async move {
            let entries = entries
                .read()
                .map_err(|err| anyhow::anyhow!("memory store read lock poisoned: {err}"))?;
            Ok(keys.iter().map(|key| entries.get(key).cloned()).collect())
        }
    }

    fn delete(&self, key: &str) -> impl Future<Output = Result<(), anyhow::Error>> + Send {
        let key = key.to_owned();
        let entries = Arc::clone(&self.entries);

        async move {
            let mut entries = entries
                .write()
                .map_err(|err| anyhow::anyhow!("memory store write lock poisoned: {err}"))?;
            entries.remove(&key);
            Ok(())
        }
    }

    fn delete_range(&self, prefix: &str) -> impl Future<Output = Result<(), anyhow::Error>> + Send {
        let prefix = prefix.to_owned();
        let entries = Arc::clone(&self.entries);

        async move {
            let mut entries = entries
                .write()
                .map_err(|err| anyhow::anyhow!("memory store write lock poisoned: {err}"))?;
            entries.retain(|key, _| !key.starts_with(&prefix));
            Ok(())
        }
    }

    fn list_range(
        &self,
        prefix: &str,
        start_after: Option<&str>,
        limit: usize,
    ) -> impl Future<Output = Result<Vec<(String, Vec<u8>)>, anyhow::Error>> + Send {
        let prefix = prefix.to_owned();
        let start_after = start_after.map(str::to_owned);
        let entries = Arc::clone(&self.entries);

        async move {
            let entries = entries
                .read()
                .map_err(|err| anyhow::anyhow!("memory store read lock poisoned: {err}"))?;

            let values = entries
                .iter()
                .filter(|(key, _)| key.starts_with(&prefix))
                .filter(|(key, _)| {
                    start_after
                        .as_ref()
                        .is_none_or(|start_after| *key > start_after)
                })
                .take(limit)
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect();

            Ok(values)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MemoryStore;
    use crate::store::StorageProvider;

    #[tokio::test]
    async fn stores_and_fetches_values() {
        let store = MemoryStore::new();

        store.put("a", b"one").await.unwrap();
        store
            .put_many(&[("b", b"two"), ("c", b"three")])
            .await
            .unwrap();

        assert_eq!(store.get("a").await.unwrap(), Some(b"one".to_vec()));
        assert_eq!(
            store.get_many(&["a", "missing", "c"]).await.unwrap(),
            vec![Some(b"one".to_vec()), None, Some(b"three".to_vec())]
        );
    }

    #[tokio::test]
    async fn lists_prefixes_in_key_order_after_start_key() {
        let store = MemoryStore::new();

        store
            .put_many(&[
                ("workflow/3", b"three"),
                ("workflow/1", b"one"),
                ("other/1", b"other"),
                ("workflow/2", b"two"),
            ])
            .await
            .unwrap();

        assert_eq!(
            store
                .list_range("workflow/", Some("workflow/1"), 2)
                .await
                .unwrap(),
            vec![
                ("workflow/2".to_owned(), b"two".to_vec()),
                ("workflow/3".to_owned(), b"three".to_vec()),
            ]
        );
    }

    #[tokio::test]
    async fn deletes_single_keys_and_prefix_ranges() {
        let store = MemoryStore::new();

        store
            .put_many(&[("run/1", b"one"), ("run/2", b"two"), ("workflow/1", b"wf")])
            .await
            .unwrap();

        store.delete("workflow/1").await.unwrap();
        store.delete_range("run/").await.unwrap();

        assert_eq!(store.get("workflow/1").await.unwrap(), None);
        assert!(store.list_range("run/", None, 10).await.unwrap().is_empty());
    }
}
