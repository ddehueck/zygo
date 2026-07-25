use serde_json::Value;

/// A storage-agnostic persistence interface.
///
/// Implementors provide the primitives the engine needs to durably store and
/// retrieve JSON values keyed by strings.
///
/// Designed to map naturally onto S3, RocksDB, Postgres, the local filesystem,
/// or any other key-value backend.
///
/// Cloning a provider must create a cheap handle to the same underlying store.
pub trait StorageProvider: Clone + Send + Sync + 'static {
    /// Put multiple key/value pairs in one operation.
    ///
    /// This must commit **atomically** so every entry is visible after
    /// success or none are.
    fn put(
        &self,
        entries: &[(&str, &Value)],
    ) -> impl Future<Output = Result<(), anyhow::Error>> + Send;

    fn get(&self, key: &str) -> impl Future<Output = Result<Option<Value>, anyhow::Error>> + Send;

    fn get_many(
        &self,
        keys: &[&str],
    ) -> impl Future<Output = Result<Vec<Option<Value>>, anyhow::Error>> + Send;
}
