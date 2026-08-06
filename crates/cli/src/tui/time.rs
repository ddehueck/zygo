use std::time::Duration;

pub fn format_duration(duration: Duration) -> String {
    let seconds = duration.as_secs();
    let (value, unit) = if seconds < 60 {
        (seconds, "second")
    } else if seconds < 60 * 60 {
        (seconds / 60, "minute")
    } else {
        (seconds / (60 * 60), "hour")
    };
    let plural = if value == 1 { "" } else { "s" };

    format!("{value} {unit}{plural}")
}

#[cfg(test)]
mod tests {
    use super::format_duration;
    use std::time::Duration;

    #[test]
    fn formats_seconds() {
        assert_eq!(format_duration(Duration::from_secs(0)), "0 seconds");
        assert_eq!(format_duration(Duration::from_secs(1)), "1 second");
        assert_eq!(format_duration(Duration::from_secs(59)), "59 seconds");
    }

    #[test]
    fn formats_minutes() {
        assert_eq!(format_duration(Duration::from_secs(60)), "1 minute");
        assert_eq!(format_duration(Duration::from_secs(3_599)), "59 minutes");
    }

    #[test]
    fn formats_hours() {
        assert_eq!(format_duration(Duration::from_secs(3_600)), "1 hour");
        assert_eq!(format_duration(Duration::from_secs(7_200)), "2 hours");
    }
}
