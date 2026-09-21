use std::time::Duration;

/// Configuration for the local worker pool.
pub struct ZygoConfig {
    pub num_workers: usize,
}

pub const DEFAULT_DATABASE_BUSY_TIMEOUT: Duration = Duration::from_secs(5);

pub struct ZygoLocalConfig {
    pub base: ZygoConfig,
    pub database_busy_timeout: Duration,
}
