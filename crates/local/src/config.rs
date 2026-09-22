use std::time::Duration;

pub const DEFAULT_DATABASE_BUSY_TIMEOUT: Duration = Duration::from_secs(5);

pub struct ZygoLocalConfig {
    pub database_busy_timeout: Duration,
}

#[derive(Clone, Copy, Debug)]
pub struct RunOptions {
    pub num_workers: usize,
    pub disable_cache: bool,
}
