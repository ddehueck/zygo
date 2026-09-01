use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::Duration;

use tokio::sync::Notify;

use crate::Repos;
use crate::sync::error::Result;
use crate::sync::schema::Delta;

const SYNC_TABLES: [&str; 2] = ["workflow_runs", "workflow_run_summary"];

#[derive(Clone)]
pub struct SyncSubscription {
    repos: Repos,
    /// The last confirmed change ID for this subscription from the CDC table.
    last_confirmed_change_id: Arc<AtomicI64>,
    /// How often to poll the CDC table for changes, in milliseconds.
    poll_interval_ms: usize,
    /// The notify object used to send a signal when changes are available.
    notify: Arc<Notify>,
}

impl SyncSubscription {
    pub fn new(repos: Repos) -> Self {
        Self {
            repos,
            last_confirmed_change_id: Arc::new(AtomicI64::new(0)),
            poll_interval_ms: 200,
            notify: Arc::new(Notify::new()),
        }
    }

    pub async fn get_deltas(&self, since: i64, max_deltas: usize) -> Result<Vec<Delta>> {
        let last_confirmed_change_id = self.last_confirmed_change_id.load(Ordering::Acquire);
        if since != last_confirmed_change_id {
            return Ok(vec![Delta::Resync]);
        }

        let through_change_id = since + max_deltas as i64;
        let cdc_rows = self
            .repos
            .cdc
            .list_between(since, through_change_id)
            .await?;

        Ok(cdc_rows
            .into_iter()
            .filter(|row| SYNC_TABLES.contains(&row.table_name.as_str()))
            .map(Delta::try_from)
            .collect::<Result<Vec<_>>>()?)
    }

    pub async fn wait_for_changes(&self) {
        self.notify.notified().await;
    }

    /// Spawns a Tokio task that polls the CDC table for changes and notifies subscribers.
    pub async fn spawn(&self) {
        let cdc = self.repos.cdc.clone();
        let last_confirmed_change_id = Arc::clone(&self.last_confirmed_change_id);
        let notify = Arc::clone(&self.notify);
        let poll_interval = self.poll_interval();
        let table_names = sync_table_names();

        tokio::spawn(async move {
            loop {
                let change_id = last_confirmed_change_id.load(Ordering::Acquire);
                if matches!(
                    cdc.has_changes_after(change_id, &table_names).await,
                    Ok(true)
                ) {
                    notify.notify_one();
                }

                tokio::time::sleep(poll_interval).await;
            }
        });
    }

    pub fn set_last_confirmed_change_id(&self, change_id: i64) {
        self.last_confirmed_change_id
            .store(change_id, Ordering::Release);
    }

    fn poll_interval(&self) -> Duration {
        Duration::from_millis(self.poll_interval_ms as u64)
    }
}

fn sync_table_names() -> Vec<String> {
    SYNC_TABLES
        .iter()
        .map(|table| (*table).to_owned())
        .collect()
}
