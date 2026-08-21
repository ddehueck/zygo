use std::{sync::Arc, time::Duration};

use tokio::sync::Mutex;
use turso::{Builder, Connection};

use super::{error::Result, migrations};

#[derive(Clone)]
pub struct Db {
    pub connection: Arc<Mutex<Connection>>,
}

impl Db {
    pub async fn open(path: &str, busy_timeout: Duration) -> Result<Self> {
        let database = Builder::new_local(path)
            .experimental_multiprocess_wal(true)
            .build()
            .await?;

        let mut connection = database.connect()?;
        connection.busy_timeout(busy_timeout)?;
        connection.pragma_update("foreign_keys", 1).await?;

        migrations::migrate(&mut connection).await?;

        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }
}
