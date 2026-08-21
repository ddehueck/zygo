use std::error::Error;

use local::{DEFAULT_DATABASE_BUSY_TIMEOUT, ZygoLocalConfig, ZygoLocalService};
use zygo_core::ZygoConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _service = ZygoLocalService::new(ZygoLocalConfig {
        base: ZygoConfig::new(1),
        database_busy_timeout: DEFAULT_DATABASE_BUSY_TIMEOUT,
    })
    .await?;
    let path = ZygoLocalService::database_path()?;

    println!("local database ready at {}", path.display());

    Ok(())
}
