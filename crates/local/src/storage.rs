use crate::KvRepository;
use anyhow::Result;
use serde_json::Value;
use zygo_core::store::{StorageProvider, StoreKey};

impl StorageProvider for KvRepository {
    async fn put(&self, entries: &[(StoreKey, Value)]) -> Result<()> {
        let entries = entries
            .iter()
            .map(|(key, value)| (key.as_str(), value))
            .collect::<Vec<_>>();
        self.upsert_many(&entries).await?;
        Ok(())
    }

    async fn get(&self, key: &StoreKey) -> Result<Option<Value>> {
        Ok(self
            .get_by_key(key.as_str())
            .await?
            .map(|entry| entry.value))
    }

    async fn get_many(&self, keys: &[StoreKey]) -> Result<Vec<Option<Value>>> {
        let mut values = Vec::with_capacity(keys.len());
        for key in keys {
            values.push(self.get(key).await?);
        }

        Ok(values)
    }
}
