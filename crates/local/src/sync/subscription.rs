use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::Duration;

use tokio::sync::Notify;

use crate::Repos;
use crate::sync::batch::DeltaBatch;
use crate::sync::error::Result;
use crate::sync::schema::{Delta, SyncEntity};

const SYNC_TABLES: [&str; 3] = ["workflow_runs", "job_runs", "tag_associations"];

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

    pub async fn load_last_change_id(&self) -> i64 {
        // load from cdc repo for use when creating a new subscription as
        // the rest will come from the snapshot retrieval of the client.
        let last_change_id = self.repos.cdc.last_change_id().await.unwrap_or(0);
        self.last_confirmed_change_id
            .store(last_change_id, Ordering::Release);
        last_change_id
    }

    pub async fn next_delta_batch(&self, max_deltas: usize) -> Result<DeltaBatch> {
        let last_confirmed_change_id = self.last_confirmed_change_id.load(Ordering::Acquire);
        let through_change_id = last_confirmed_change_id + max_deltas as i64;

        let cdc_rows = self
            .repos
            .cdc
            .list_between(last_confirmed_change_id, through_change_id)
            .await?;

        // The query boundary is only a batching limit. Confirm the highest CDC
        // row actually fetched so a sparse CDC range cannot skip later rows.
        let max_change_id = cdc_rows
            .iter()
            .map(|row| row.change_id)
            .max()
            .unwrap_or(last_confirmed_change_id);

        let deltas = cdc_rows
            .into_iter()
            .filter(|row| SYNC_TABLES.contains(&row.table_name.as_str()))
            .map(Delta::try_from)
            .collect::<Result<Vec<_>>>()?;

        let mut hydrated_deltas = Vec::with_capacity(deltas.len());
        for delta in deltas {
            hydrated_deltas.push(self.hydrate_tag_delta(delta).await?);
        }

        Ok(DeltaBatch::new(max_change_id, hydrated_deltas))
    }

    pub async fn wait_for_changes(&self) {
        self.notify.notified().await;
    }

    async fn hydrate_tag_delta(&self, delta: Delta) -> Result<Delta> {
        let Delta::Upsert {
            entity: SyncEntity::Tag,
            id,
            ..
        } = &delta
        else {
            return Ok(delta);
        };

        let id = id
            .parse::<i64>()
            .map_err(|_| super::error::Error::InvalidTagId { id: id.clone() })?;

        let Some(tag) = self.repos.tags.get_by_id(id).await? else {
            // The association may have been deleted after CDC was read. A
            // delete is the correct current-state update for the client.
            return Ok(Delta::Delete {
                entity: SyncEntity::Tag,
                id: id.to_string(),
            });
        };

        Ok(Delta::Upsert {
            entity: SyncEntity::Tag,
            id: tag.id.to_string(),
            data: serde_json::json!({
                "id": tag.id.to_string(),
                "workflow_run_id": tag.workflow_run_id,
                "key": tag.key,
                "value": tag.value,
                "created_at": tag.created_at,
            }),
        })
    }

    /// Spawns a Tokio task that polls the CDC table for changes and notifies subscribers.
    pub async fn spawn(&self) {
        let cdc = self.repos.cdc.clone();
        let last_confirmed_change_id = Arc::downgrade(&self.last_confirmed_change_id);
        let notify = Arc::downgrade(&self.notify);
        let poll_interval = self.poll_interval();
        let table_names = sync_table_names();

        tokio::spawn(async move {
            loop {
                // The subscription is owned by the command. Weak references
                // let this detached poller stop after a reload or channel
                // failure instead of surviving every sync invocation.
                let Some(last_confirmed_change_id) = last_confirmed_change_id.upgrade() else {
                    break;
                };
                let Some(notify) = notify.upgrade() else {
                    break;
                };

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
