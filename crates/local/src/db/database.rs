use std::{sync::Arc, time::Duration};

use tokio::sync::Mutex;
use turso::{Builder, Connection};

use super::{DbResult, migrations};

#[derive(Clone)]
pub struct Db {
    pub connection: Arc<Mutex<Connection>>,
}

impl Db {
    pub async fn open(path: &str, busy_timeout: Duration) -> DbResult<Self> {
        let database = Builder::new_local(path)
            .experimental_multiprocess_wal(true)
            .build()
            .await?;
        let mut connection = database.connect()?;

        // todo: do this in the not open path.
        // there should be an explicit lock for this maybe?
        migrations::migrate(&mut connection, busy_timeout).await?;

        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }
}
