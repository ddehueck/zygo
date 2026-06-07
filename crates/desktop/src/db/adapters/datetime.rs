use std::time::SystemTime;

/// Convert chrono NaiveDateTime to SystemTime
pub fn naive_datetime_to_system_time(dt: chrono::NaiveDateTime) -> SystemTime {
    use chrono::{TimeZone, Utc};
    let datetime = Utc.from_utc_datetime(&dt);
    SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(datetime.timestamp() as u64)
}

/// Convert SystemTime to chrono NaiveDateTime
pub fn system_time_to_naive_datetime(st: SystemTime) -> chrono::NaiveDateTime {
    let duration = st.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
    chrono::DateTime::from_timestamp(duration.as_secs() as i64, duration.subsec_nanos())
        .map(|dt| dt.naive_utc())
        .unwrap_or_else(|| chrono::Utc::now().naive_utc())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_conversion() {
        let now = SystemTime::now();
        let naive = system_time_to_naive_datetime(now);
        let back = naive_datetime_to_system_time(naive);
        
        // Allow 1 second tolerance due to sub-second truncation
        let diff = now.duration_since(back).unwrap_or_default();
        assert!(diff.as_secs() <= 1);
    }
}

