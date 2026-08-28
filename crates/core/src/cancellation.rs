use std::future::Future;

use tokio::task::JoinHandle;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

#[derive(Clone)]
pub struct CancellationGroup {
    token: CancellationToken,
    tasks: TaskTracker,
}

impl CancellationGroup {
    pub fn new() -> Self {
        Self {
            token: CancellationToken::new(),
            tasks: TaskTracker::new(),
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.token.is_cancelled()
    }

    pub async fn cancelled(&self) {
        self.token.cancelled().await;
    }

    pub fn spawn<F>(&self, task: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.tasks.spawn(task)
    }

    pub async fn cancel_and_wait(&self) {
        self.tasks.close();
        self.token.cancel();
        self.tasks.wait().await;
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use super::CancellationGroup;

    #[tokio::test]
    async fn cancellation_waits_for_tracked_tasks_to_finish_cleanup() {
        let group = CancellationGroup::new();
        let completed = Arc::new(AtomicUsize::new(0));

        for _ in 0..3 {
            let task_group = group.clone();
            let completed = completed.clone();
            drop(group.spawn(async move {
                task_group.cancelled().await;
                tokio::task::yield_now().await;
                completed.fetch_add(1, Ordering::SeqCst);
            }));
        }

        group.cancel_and_wait().await;
        assert_eq!(completed.load(Ordering::SeqCst), 3);

        // Cancellation is idempotent and remains safe after all tasks exit.
        group.cancel_and_wait().await;
    }
}
