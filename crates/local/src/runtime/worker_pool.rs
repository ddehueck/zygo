use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::{Result, anyhow};
use tokio::sync::{Mutex, mpsc};
use tokio::task::{JoinError, JoinHandle, JoinSet};

type JobFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;
type CancelJob = Box<dyn FnOnce() + Send + 'static>;

pub struct WorkerPool {
    inner: Arc<WorkerPoolInner>,
}

struct WorkerPoolInner {
    sender: mpsc::UnboundedSender<Command>,
    closed: Arc<AtomicBool>,
    scheduler: Mutex<Option<JoinHandle<Result<()>>>>,
}

enum Command {
    Enqueue(ScheduledJob),
    Cancel,
}

struct ScheduledJob {
    future: JobFuture,
    cancel: CancelJob,
}

impl Clone for WorkerPool {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl WorkerPool {
    pub fn new(num_workers: usize) -> Self {
        assert!(
            num_workers > 0,
            "local runtime requires at least one worker"
        );

        let (sender, receiver) = mpsc::unbounded_channel();
        let closed = Arc::new(AtomicBool::new(false));
        let scheduler = tokio::spawn(run_scheduler(receiver, num_workers, closed.clone()));
        Self {
            inner: Arc::new(WorkerPoolInner {
                sender,
                closed,
                scheduler: Mutex::new(Some(scheduler)),
            }),
        }
    }

    pub fn is_closed(&self) -> bool {
        self.inner.closed.load(Ordering::SeqCst)
    }

    pub fn enqueue<F, C>(&self, future: F, cancel: C) -> Result<()>
    where
        F: Future<Output = ()> + Send + 'static,
        C: FnOnce() + Send + 'static,
    {
        if self.is_closed() {
            return Err(anyhow!("local worker pool is closed"));
        }

        self.inner
            .sender
            .send(Command::Enqueue(ScheduledJob {
                future: Box::pin(future),
                cancel: Box::new(cancel),
            }))
            .map_err(|_| anyhow!("local worker pool is closed"))
    }

    pub fn close(&self) {
        if !self.inner.closed.swap(true, Ordering::SeqCst) {
            let _ = self.inner.sender.send(Command::Cancel);
        }
    }

    pub async fn join(&self) -> Result<()> {
        let mut scheduler = self.inner.scheduler.lock().await;
        let Some(scheduler) = scheduler.take() else {
            return Ok(());
        };

        scheduler
            .await
            .map_err(|error| anyhow!("local worker scheduler failed: {error}"))?
    }
}

async fn run_scheduler(
    mut receiver: mpsc::UnboundedReceiver<Command>,
    num_workers: usize,
    closed: Arc<AtomicBool>,
) -> Result<()> {
    let mut queue: VecDeque<ScheduledJob> = VecDeque::new();
    let mut active = JoinSet::new();
    let mut first_error = None;

    loop {
        if closed.load(Ordering::SeqCst) {
            receiver.close();
            cancel_queued(&mut queue, &mut receiver);
            while let Some(result) = active.join_next().await {
                record_join_error(result, &mut first_error);
            }
            return match first_error {
                Some(error) => Err(error),
                None => Ok(()),
            };
        }

        while active.len() < num_workers && !closed.load(Ordering::SeqCst) {
            let Some(job) = queue.pop_front() else {
                break;
            };
            active.spawn(job.future);
        }

        tokio::select! {
            biased;
            result = active.join_next(), if !active.is_empty() => {
                if let Some(result) = result {
                    record_join_error(result, &mut first_error);
                }
            }
            command = receiver.recv() => match command {
                Some(Command::Enqueue(job)) => queue.push_back(job),
                Some(Command::Cancel) | None => {
                    closed.store(true, Ordering::SeqCst);
                }
            },
        }
    }
}

fn cancel_queued(
    queue: &mut VecDeque<ScheduledJob>,
    receiver: &mut mpsc::UnboundedReceiver<Command>,
) {
    for job in queue.drain(..) {
        (job.cancel)();
    }
    while let Ok(command) = receiver.try_recv() {
        if let Command::Enqueue(job) = command {
            (job.cancel)();
        }
    }
}

fn record_join_error(
    result: std::result::Result<(), JoinError>,
    first_error: &mut Option<anyhow::Error>,
) {
    if let Err(error) = result {
        first_error.get_or_insert_with(|| anyhow!("local worker task failed: {error}"));
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicBool, Ordering};

    use tokio::sync::mpsc::error::TryRecvError;
    use tokio::sync::{mpsc, oneshot, watch};

    use super::WorkerPool;

    #[tokio::test]
    async fn starts_queued_jobs_only_when_capacity_is_available() {
        let pool = WorkerPool::new(1);
        let (started_tx, mut started_rx) = mpsc::unbounded_channel();
        let (release_tx, release_rx) = oneshot::channel();

        let first_started = started_tx.clone();
        pool.enqueue(
            async move {
                first_started.send(1).unwrap();
                let _ = release_rx.await;
            },
            || {},
        )
        .unwrap();
        pool.enqueue(
            async move {
                started_tx.send(2).unwrap();
            },
            || {},
        )
        .unwrap();

        assert_eq!(started_rx.recv().await, Some(1));
        tokio::task::yield_now().await;
        assert_eq!(started_rx.try_recv(), Err(TryRecvError::Empty));

        release_tx.send(()).unwrap();
        assert_eq!(started_rx.recv().await, Some(2));
        pool.close();
        pool.join().await.unwrap();
    }

    #[tokio::test]
    async fn cancellation_drains_queued_jobs_without_starting_them() {
        let pool = WorkerPool::new(1);
        let (started_tx, started_rx) = oneshot::channel();
        let (cancel_tx, mut cancel_rx) = watch::channel(false);
        let queued_started = std::sync::Arc::new(AtomicBool::new(false));
        let queued_cancelled = std::sync::Arc::new(AtomicBool::new(false));

        pool.enqueue(
            async move {
                started_tx.send(()).unwrap();
                while !*cancel_rx.borrow_and_update() {
                    if cancel_rx.changed().await.is_err() {
                        break;
                    }
                }
            },
            || {},
        )
        .unwrap();

        let started = queued_started.clone();
        let cancelled = queued_cancelled.clone();
        pool.enqueue(
            async move {
                started.store(true, Ordering::SeqCst);
            },
            move || cancelled.store(true, Ordering::SeqCst),
        )
        .unwrap();

        started_rx.await.unwrap();
        pool.close();
        cancel_tx.send_replace(true);
        pool.join().await.unwrap();

        assert!(!queued_started.load(Ordering::SeqCst));
        assert!(queued_cancelled.load(Ordering::SeqCst));
    }
}
