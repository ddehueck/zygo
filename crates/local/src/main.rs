use std::error::Error;

use local::ZygoLocalService;
use zygo_core::ZygoConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _service = ZygoLocalService::new(ZygoConfig::new(1)).await?;
    let path = ZygoLocalService::database_path()?;

    println!("local database ready at {}", path.display());

    Ok(())
}
