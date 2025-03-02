/// Format a git time to a readable string
pub fn format_time(time: &git2::Time) -> String {
    let seconds = time.seconds();
    let offset = time.offset_minutes();
    
    let timestamp = seconds + (offset as i64 * 60);
    format!("{}", timestamp)
}
