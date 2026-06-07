/// A storage-agnostic persistence interface.
///
/// Implementors provide the primitives the engine needs to durably
/// store and retrieve opaque byte records keyed by string. The engine
/// builds higher-level concepts (typed collections, event streams) on
/// top of these operations.
///
/// Designed to map naturally onto S3, RocksDB, Postgres, the local
/// filesystem, or any other ordered key-value store.
pub trait StorageProvider: Send + Sync {
    fn put(
        &self,
        key: &str,
        value: &[u8],
    ) -> impl Future<Output = Result<(), anyhow::Error>> + Send;

    /// Put multiple key/value pairs in one operation.
    ///
    /// Contract: this must commit **atomically** — either every entry is
    /// visible together after success, or none are (same semantics as a DB
    /// transaction or `WriteBatch::commit`).
    fn put_many<'a>(
        &'a self,
        entries: &'a [(&'a str, &'a [u8])],
    ) -> impl Future<Output = Result<(), anyhow::Error>> + Send;

    fn get(&self, key: &str)
    -> impl Future<Output = Result<Option<Vec<u8>>, anyhow::Error>> + Send;

    fn get_many(
        &self,
        keys: &[&str],
    ) -> impl Future<Output = Result<Vec<Option<Vec<u8>>>, anyhow::Error>> + Send;

    /// Delete a single key. A no-op if the key does not exist.
    fn delete(&self, key: &str) -> impl Future<Output = Result<(), anyhow::Error>> + Send;

    /// Delete every key with the given prefix. Implementations SHOULD make
    /// this efficient (e.g. RocksDB `delete_range`, S3 bulk delete,
    /// `DELETE WHERE key LIKE 'prefix%'`), but may fall back to a scan.
    fn delete_range(&self, prefix: &str) -> impl Future<Output = Result<(), anyhow::Error>> + Send;

    fn list_range(
        &self,
        prefix: &str,
        start_after: Option<&str>,
        limit: usize,
    ) -> impl Future<Output = Result<Vec<(String, Vec<u8>)>, anyhow::Error>> + Send;
}
