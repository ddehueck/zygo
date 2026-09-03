use std::time::SystemTime;

use chrono::{DateTime, Utc};

const DATABASE_TIMESTAMP_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

pub fn format_database_timestamp(timestamp: SystemTime) -> String {
    DateTime::<Utc>::from(timestamp)
        .format(DATABASE_TIMESTAMP_FORMAT)
        .to_string()
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, UNIX_EPOCH};

    use super::format_database_timestamp;

    #[test]
    fn formats_timestamp_like_sqlite_current_timestamp() {
        let timestamp = UNIX_EPOCH + Duration::from_millis(1_787_191_601_927);

        assert_eq!(format_database_timestamp(timestamp), "2026-08-20 02:06:41");
    }
}
