use std::time::{Duration, SystemTime};

use chrono::{Datelike, Local, Timelike};

pub fn format_time_ago(timestamp: SystemTime) -> String {
    let elapsed = SystemTime::now()
        .duration_since(timestamp)
        .unwrap_or(Duration::ZERO);

    format_duration_ago(elapsed)
}

pub fn format_local_datetime(timestamp: SystemTime) -> String {
    let datetime = chrono::DateTime::<Local>::from(timestamp);
    let hour = match datetime.hour() % 12 {
        0 => 12,
        hour => hour,
    };
    let meridiem = if datetime.hour() < 12 { "am" } else { "pm" };
    let day = datetime.day();

    format!(
        "{hour}:{:02}{meridiem} on {} {day}{}",
        datetime.minute(),
        datetime.format("%b"),
        ordinal_suffix(day)
    )
}

fn ordinal_suffix(day: u32) -> &'static str {
    match day % 100 {
        11..=13 => "th",
        _ => match day % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        },
    }
}

fn format_duration_ago(duration: Duration) -> String {
    let seconds = duration.as_secs();
    let (value, unit) = if seconds < 60 {
        (seconds, "second")
    } else if seconds < 60 * 60 {
        (seconds / 60, "minute")
    } else {
        (seconds / (60 * 60), "hour")
    };
    let plural = if value == 1 { "" } else { "s" };

    format!("{value} {unit}{plural} ago")
}

#[cfg(test)]
mod tests {
    use super::{format_duration_ago, format_local_datetime, ordinal_suffix};
    use chrono::{Local, TimeZone};
    use std::time::{Duration, SystemTime};

    #[test]
    fn formats_local_datetimes() {
        let datetime = Local
            .with_ymd_and_hms(2026, 8, 5, 14, 31, 0)
            .single()
            .expect("test date must be valid in the local timezone");
        let timestamp: SystemTime = datetime.into();

        assert_eq!(format_local_datetime(timestamp), "2:31pm on Aug 5th");
    }

    #[test]
    fn formats_ordinal_suffixes() {
        assert_eq!(ordinal_suffix(1), "st");
        assert_eq!(ordinal_suffix(2), "nd");
        assert_eq!(ordinal_suffix(3), "rd");
        assert_eq!(ordinal_suffix(11), "th");
        assert_eq!(ordinal_suffix(12), "th");
        assert_eq!(ordinal_suffix(13), "th");
        assert_eq!(ordinal_suffix(21), "st");
        assert_eq!(ordinal_suffix(22), "nd");
        assert_eq!(ordinal_suffix(23), "rd");
    }

    #[test]
    fn formats_seconds() {
        assert_eq!(format_duration_ago(Duration::from_secs(0)), "0 seconds ago");
        assert_eq!(format_duration_ago(Duration::from_secs(1)), "1 second ago");
        assert_eq!(
            format_duration_ago(Duration::from_secs(59)),
            "59 seconds ago"
        );
    }

    #[test]
    fn formats_minutes() {
        assert_eq!(format_duration_ago(Duration::from_secs(60)), "1 minute ago");
        assert_eq!(
            format_duration_ago(Duration::from_secs(3_599)),
            "59 minutes ago"
        );
    }

    #[test]
    fn formats_hours() {
        assert_eq!(
            format_duration_ago(Duration::from_secs(3_600)),
            "1 hour ago"
        );
        assert_eq!(
            format_duration_ago(Duration::from_secs(7_200)),
            "2 hours ago"
        );
    }
}
