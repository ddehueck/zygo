use std::time::Duration;

use turso::Connection;
use turso::transaction::TransactionBehavior;

pub const MIGRATIONS: &[(i64, &str)] = &[
    (1, include_str!("0001_initial.sql")),
    (2, include_str!("0002_kv.sql")),
];
pub const DEFAULT_BUSY_TIMEOUT: Duration = Duration::from_secs(5);

pub struct MigrationRunner<'conn> {
    connection: &'conn mut Connection,
    busy_timeout: Duration,
}

impl<'conn> MigrationRunner<'conn> {
    pub fn new(connection: &'conn mut Connection) -> Self {
        Self {
            connection,
            busy_timeout: DEFAULT_BUSY_TIMEOUT,
        }
    }

    pub fn with_busy_timeout(mut self, busy_timeout: Duration) -> Self {
        self.busy_timeout = busy_timeout;
        self
    }

    pub async fn run(self) -> turso::Result<()> {
        self.connection.busy_timeout(self.busy_timeout)?;
        self.connection.pragma_update("foreign_keys", 1).await?;

        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .await?;

        let current_version = MigrationRunner::current_version(&tx).await?;

        for &(version, sql) in MIGRATIONS {
            if version <= current_version {
                continue;
            }

            tx.execute_batch(sql).await?;
            tx.pragma_update("user_version", version).await?;
        }

        tx.commit().await
    }

    async fn current_version(conn: &Connection) -> turso::Result<i64> {
        let mut rows = conn.query("PRAGMA user_version", ()).await?;
        let row = rows
            .next()
            .await?
            .ok_or(turso::Error::QueryReturnedNoRows)?;
        row.get(0)
    }
}

pub async fn migrate(conn: &mut Connection) -> turso::Result<()> {
    MigrationRunner::new(conn).run().await
}
