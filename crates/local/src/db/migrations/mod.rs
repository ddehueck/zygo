use turso::{Connection, transaction::TransactionBehavior};

use super::error::Result;

const MIGRATIONS: &[(i64, &str)] = &[
    (1, include_str!("0001_initial.sql")),
    (2, include_str!("0002_kv.sql")),
    (3, include_str!("0003_logs.sql")),
];

pub async fn migrate(connection: &mut Connection) -> Result<()> {
    let tx = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .await?;

    let current_version = current_version(&tx).await?;
    for &(version, sql) in MIGRATIONS {
        if version <= current_version {
            continue;
        }

        tx.execute_batch(sql).await?;
        tx.pragma_update("user_version", version).await?;
    }

    tx.commit().await?;
    Ok(())
}

async fn current_version(conn: &Connection) -> Result<i64> {
    let mut rows = conn.query("PRAGMA user_version", ()).await?;
    let row = rows
        .next()
        .await?
        .ok_or(turso::Error::QueryReturnedNoRows)?;
    Ok(row.get(0)?)
}

#[cfg(test)]
mod tests {
    use super::{current_version, migrate};

    #[tokio::test]
    async fn logs_migration_supports_ordered_reads_and_search() -> anyhow::Result<()> {
        let database = turso::Builder::new_local(":memory:")
            .experimental_index_method(true)
            .build()
            .await?;
        let mut connection = database.connect()?;
        connection.pragma_update("foreign_keys", 1).await?;

        connection
            .execute_batch(concat!(
                include_str!("0001_initial.sql"),
                include_str!("0002_kv.sql")
            ))
            .await?;
        connection.pragma_update("user_version", 2).await?;
        migrate(&mut connection).await?;
        migrate(&mut connection).await?;
        assert_eq!(current_version(&connection).await?, 3);

        connection
            .execute_batch(
                "INSERT INTO workflow_runs (id, workflow_id, content_hash, status)
                 VALUES ('workflow', 'definition', 'hash', 'running');
                 INSERT INTO job_runs (id, workflow_run_id, job_id, status)
                 VALUES ('job', 'workflow', 'first', 'running'),
                        ('other', 'workflow', 'second', 'running');",
            )
            .await?;

        let tx = connection.transaction().await?;
        tx.execute(
            "INSERT INTO logs (job_run_id, \"order\", content)
             VALUES ('job', 2, 'connection timeout'),
                    ('job', 1, 'starting worker'),
                    ('other', 1, 'another timeout')",
            (),
        )
        .await?;
        tx.commit().await?;

        let mut rows = connection
            .query(
                "SELECT \"order\" FROM logs
                 WHERE job_run_id = 'job' AND \"order\" > 0
                 ORDER BY \"order\" LIMIT 100",
                (),
            )
            .await?;
        assert_eq!(rows.next().await?.unwrap().get::<i64>(0)?, 1);
        assert_eq!(rows.next().await?.unwrap().get::<i64>(0)?, 2);
        assert!(rows.next().await?.is_none());
        drop(rows);

        let mut rows = connection
            .query(
                "SELECT \"order\", content FROM logs
                 WHERE fts_match(content, ?1) AND job_run_id = ?2
                 ORDER BY \"order\" LIMIT 100",
                ["timeout", "job"],
            )
            .await?;
        let row = rows.next().await?.unwrap();
        assert_eq!(row.get::<i64>(0)?, 2);
        assert_eq!(row.get::<String>(1)?, "connection timeout");
        assert!(rows.next().await?.is_none());
        drop(rows);

        for sql in [
            "INSERT INTO logs VALUES ('job', 2, 'duplicate', CURRENT_TIMESTAMP)",
            "INSERT INTO logs VALUES ('job', 0, 'invalid', CURRENT_TIMESTAMP)",
            "INSERT INTO logs VALUES ('missing', 1, 'orphan', CURRENT_TIMESTAMP)",
        ] {
            let tx = connection.transaction().await?;
            assert!(tx.execute(sql, ()).await.is_err());
            tx.rollback().await?;
        }

        let tx = connection.transaction().await?;
        tx.execute("DELETE FROM job_runs WHERE id = 'job'", ())
            .await?;
        tx.commit().await?;
        let mut rows = connection
            .query(
                "SELECT job_run_id FROM logs WHERE fts_match(content, 'timeout')",
                (),
            )
            .await?;
        assert_eq!(rows.next().await?.unwrap().get::<String>(0)?, "other");
        assert!(rows.next().await?.is_none());
        Ok(())
    }
}
