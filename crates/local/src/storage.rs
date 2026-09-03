use crate::KvRepository;
use anyhow::Result;
use serde_json::Value;
use zygo_core::store::StorageProvider;

impl StorageProvider for KvRepository {
    async fn put(&self, entries: &[(&str, &Value)]) -> Result<()> {
        self.upsert_many(entries).await?;
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Option<Value>> {
        Ok(self.get_by_key(key).await?.map(|entry| entry.value))
    }

    async fn get_many(&self, keys: &[&str]) -> Result<Vec<Option<Value>>> {
        let mut values = Vec::with_capacity(keys.len());
        for key in keys {
            values.push(self.get(key).await?);
        }

        Ok(values)
    }
}
