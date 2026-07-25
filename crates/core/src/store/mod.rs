pub(crate) mod keyspace;
mod memory;
mod provider_interface;
mod write_set;

pub use keyspace::StoreKey;
pub use memory::MemoryStore;
pub use provider_interface::StorageProvider;
pub use write_set::WriteSet;
pub(crate) use write_set::WriteSetReservation;

use serde_json::Value;

#[derive(Clone)]
pub struct Store<S: StorageProvider> {
    storage: S,
}

impl<S: StorageProvider> Store<S> {
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    pub async fn put(&self, write_set: &[(StoreKey, Value)]) -> anyhow::Result<()> {
        let entries = write_set
            .iter()
            .map(|(key, value)| (key.as_str(), value))
            .collect::<Vec<_>>();

        self.storage.put(&entries).await
    }

    pub async fn get(&self, key: &StoreKey) -> anyhow::Result<Option<Value>> {
        self.storage.get(key.as_str()).await
    }

    pub async fn get_many(&self, keys: &[StoreKey]) -> anyhow::Result<Vec<Option<Value>>> {
        let keys = keys.iter().map(StoreKey::as_str).collect::<Vec<_>>();
        self.storage.get_many(&keys).await
    }
}
