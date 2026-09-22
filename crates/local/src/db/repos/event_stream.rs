use turso::params;
use turso::transaction::TransactionBehavior;

use crate::db::{Db, DbError, DbResult};

/// Database access for an ordered, run-scoped payload log.
///
/// This repository intentionally knows nothing about the payload format. Domain
/// validation and serialization belong to the event-stream adapter.
#[derive(Clone)]
pub struct EventStreamRepository {
    database: Db,
}

impl EventStreamRepository {
    pub fn new(database: Db) -> Self {
        Self { database }
    }

    /// Appends serialized payloads in order, assigning contiguous keys starting at zero.
    pub async fn append(&self, run_id: &str, payloads: Vec<String>) -> DbResult<()> {
        if payloads.is_empty() {
            return Ok(());
        }

        let mut connection = self.database.connection.lock().await;
        // Acquire the database writer lock before reading the last key, including
        // when other processes or independently opened connections append.
        let tx = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .await?;
        let result: DbResult<()> = async {
            let mut sequence: i64 = {
                let mut rows = tx
                    .query(
                        "SELECT COALESCE(MAX(sequence_id), -1) FROM event_stream WHERE workflow_run_id = ?1",
                        params![run_id],
                    )
                    .await?;
                let row = rows.next().await?.ok_or(turso::Error::QueryReturnedNoRows)?;
                row.get(0)?
            };

            for payload in payloads {
                sequence = sequence
                    .checked_add(1)
                    .ok_or_else(|| DbError::EventSequenceExhausted(run_id.to_owned()))?;
                tx.execute(
                    "INSERT INTO event_stream (workflow_run_id, sequence_id, event) VALUES (?1, ?2, ?3)",
                    params![run_id, sequence, payload],
                )
                .await?;
            }
            Ok(())
        }
        .await;
        if let Err(error) = result {
            tx.rollback().await?;
            return Err(error);
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn get(&self, run_id: &str, sequence: i64) -> DbResult<Option<String>> {
        let connection = self.database.connection.lock().await;
        let mut rows = connection
            .query(
                "SELECT event FROM event_stream WHERE workflow_run_id = ?1 AND sequence_id = ?2",
                params![run_id, sequence],
            )
            .await?;
        let Some(row) = rows.next().await? else {
            return Ok(None);
        };
        Ok(Some(row.get(0)?))
    }
}
