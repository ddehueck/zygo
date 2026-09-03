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
        // We are using multi-process WAL so the we can have the desktop app
        // and cli interact with the same database.
        //
        // NB: Watch out for the following error:
        // thread 'main' (413596) panicked at src/lib.rs:36:6: failed to start the local Zygo service: I/O error: short read on WAL frame at offset 1355512: expected 4096 bytes, got 0 note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace [ELIFECYCLE] Command failed with exit code 101. error: Recipe `dev` failed on line 6 with exit code 101
        // This means the WAL got corrupted via another process that interacted with the database concurrently, but incorrectly.
        // This happened to me while use beekeeper studio as a GUI to interact with DB locally
        // Recommended path is to use: `turso --dev path/to/zygo.db`'s http server with Outerbase.
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

        // Conditionally enabled so only stream processing which projects run state
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
