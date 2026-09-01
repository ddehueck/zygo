use std::{sync::Arc, time::Duration};

use tokio::sync::Mutex;
use turso::{Builder, Connection};

use super::{error::Result, migrations};

#[derive(Clone)]
pub struct Db {
    pub connection: Arc<Mutex<Connection>>,
    pub is_cdc_enabled: bool,
}

impl Db {
    pub async fn open(path: &str, busy_timeout: Duration, enable_cdc: bool) -> Result<Self> {
        let database = Builder::new_local(path)
            .experimental_multiprocess_wal(true)
            .build()
            .await?;

        let mut connection = database.connect()?;

        // We turn on the following settings for our db
        // - busy_timeout: the maximum time to wait for a database lock
        // - foreign_keys: enforce foreign key constraints
        // - capture_data_changes_conn: enable data change notifications
        // - we also turn on multi-process WAL support above
        connection.busy_timeout(busy_timeout)?;
        connection.pragma_update("foreign_keys", 1).await?;

        // Conditionally enabled so only stream processing which builds the summary
        // tables write to the cdc table.
        if enable_cdc {
            connection
                .execute("PRAGMA capture_data_changes_conn('after')", ())
                .await?;
        }

        migrations::migrate(&mut connection).await?;

        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
            is_cdc_enabled: enable_cdc,
        })
    }
}
