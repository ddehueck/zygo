use std::path::Path;
use std::str::FromStr;

use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};

use crate::dirs::Directories;

#[derive(Clone, Debug)]
pub struct DB {
    pool: SqlitePool,
}

impl DB {
    pub async fn new<P: AsRef<Path>>() -> Result<Self, anyhow::Error> {
        let dirs = Directories::new()?;
        dirs.ensure()?;

        let path = dirs.store_db_path();
        let url = format!("sqlite://{}?mode=rwc", path.to_string_lossy());

        let options = SqliteConnectOptions::from_str(&url)?.foreign_keys(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        // Run migrations
        sqlx::migrate!("src/db/migrations").run(&pool).await?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}
