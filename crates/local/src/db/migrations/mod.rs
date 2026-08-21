use turso::{Connection, transaction::TransactionBehavior};

use super::error::Result;

const MIGRATIONS: &[(i64, &str)] = &[
    (1, include_str!("0001_initial.sql")),
    (2, include_str!("0002_kv.sql")),
];

pub async fn migrate(connection: &mut Connection) -> Result<()> {
    migrate_once(connection).await
}

async fn migrate_once(connection: &mut Connection) -> Result<()> {
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
