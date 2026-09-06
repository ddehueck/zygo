use std::time::Duration;

use tokio::time::{Instant, sleep_until};
use zygo_core::models::JobRunId;

use crate::{DbResult, LogRow, LogsRepository};

const POLL_INTERVAL: Duration = Duration::from_millis(200);
const PAGE_SIZE: u32 = 512;

/// Pages through a job's history, then polls for newly committed output.
///
/// No background task is spawned: callers drive the watcher with `next_batch`.
/// Dropping the watcher unsubscribes. Job completion does not stop polling,
/// because a final batch may be persisted after the completion event.
pub struct LogWatcher {
    repository: LogsRepository,
    job_run_id: String,
    after_order: i64,
    next_poll: Instant,
}

impl LogWatcher {
    pub fn new(repository: LogsRepository, job_run_id: JobRunId) -> Self {
        Self {
            repository,
            job_run_id: job_run_id.to_string(),
            after_order: 0,
            next_poll: Instant::now(),
        }
    }

    /// Returns at most 512 rows, preserving their stored content and order.
    ///
    /// Full pages are followed immediately by another read; otherwise the next
    /// read waits 200 ms. An empty batch means caught up, not end of stream.
    /// Errors preserve the cursor and are retried on the same cadence.
    /// This method is cancellation-safe: the cursor advances only when a whole
    /// batch is returned, so it can be used directly in `tokio::select!`.
    pub async fn next_batch(&mut self) -> DbResult<Vec<LogRow>> {
        sleep_until(self.next_poll).await;
        let result = self
            .repository
            .list_after(&self.job_run_id, self.after_order, PAGE_SIZE)
            .await;

        self.next_poll = Instant::now() + POLL_INTERVAL;
        let rows = result?;
        if let Some(last) = rows.last() {
            self.after_order = last.order;
        }
        if rows.len() == PAGE_SIZE as usize {
            self.next_poll = Instant::now();
        }
        Ok(rows)
    }
}
