use chrono::{DateTime, TimeZone, Utc};

/// Format a git time to a readable string
pub fn format_time(time: &git2::Time) -> String {
    let seconds = time.seconds();
    let offset = time.offset_minutes();
    
    let timestamp = seconds + (offset as i64 * 60);
    format!("{}", timestamp)
}

/// Format a timestamp to a readable string
pub fn format_timestamp(timestamp: i64) -> String {
    let datetime: DateTime<Utc> = Utc.timestamp_opt(timestamp, 0).unwrap();
    format!("{}", datetime.format("%d %b %Y %H:%M:%S"))
}
