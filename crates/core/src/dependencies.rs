use serde_json::Value;

use crate::models::{JobRunSource, WorkflowRunId};
use crate::store::StoreKey;

#[derive(Clone)]
pub struct Dependencies<S, L> {
    store: S,
    log_writer: L,
}

impl<S, L> Dependencies<S, L> {
    pub fn new(store: S, log_writer: L) -> Self {
        Self { store, log_writer }
    }
}

/// Access to the application's dependency namespaces.
pub trait AppDeps: Clone + Send + Sync + 'static {
    type Store: StorageProvider;
    type LogWriter: self::LogWriter;

    fn store(&self) -> &Self::Store;
    fn log_writer(&self) -> &Self::LogWriter;
}

impl<S, L> AppDeps for Dependencies<S, L>
where
    S: StorageProvider,
    L: LogWriter + Clone + Send + Sync + 'static,
{
    type Store = S;
    type LogWriter = L;

    fn store(&self) -> &S {
        &self.store
    }

    fn log_writer(&self) -> &L {
        &self.log_writer
    }
}

/// A storage-agnostic persistence interface.
///
/// Implementors provide the primitives the engine needs to durably store and
/// retrieve JSON values keyed by `StoreKey`s.
pub trait StorageProvider: Clone + Send + Sync + 'static {
    /// Put multiple key/value pairs in one operation.
    ///
    /// This must commit **atomically** so every entry is visible after
    /// success or none are.
    fn put(
        &self,
        entries: &[(StoreKey, Value)],
    ) -> impl Future<Output = Result<(), anyhow::Error>> + Send;

    fn get(
        &self,
        key: &StoreKey,
    ) -> impl Future<Output = Result<Option<Value>, anyhow::Error>> + Send;

    fn get_many(
        &self,
        keys: &[StoreKey],
    ) -> impl Future<Output = Result<Vec<Option<Value>>, anyhow::Error>> + Send;
}

/// Writes job output to the application's log sink.
///
/// Implementations should make each call behave like `Write::write_all`: the
/// future succeeds only after all bytes have been accepted by the sink.
/// `write_all` is the durability boundary for a log entry. The runner supplies
/// complete newline-delimited lines, or the final unterminated line at EOF.
/// Run and job identity let sinks persist output without relying on UI projection.
pub trait LogWriter: Clone + Send + Sync + 'static {
    fn write_all(
        &self,
        workflow_run_id: &WorkflowRunId,
        source: &JobRunSource,
        bytes: &[u8],
    ) -> impl Future<Output = std::io::Result<()>> + Send;
}
