use std::time::Duration;

use zygo_core::ZygoConfig;

pub const DEFAULT_DATABASE_BUSY_TIMEOUT: Duration = Duration::from_secs(5);

pub struct ZygoLocalConfig {
    pub base: ZygoConfig,
    pub database_busy_timeout: Duration,
}
