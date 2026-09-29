//! Text formatting for values that views show.

/// A coarse relative age like the Docker CLI prints, for example "3 hours ago".
/// Both arguments are Unix timestamps in seconds.
pub fn age_label(created: i64, now: i64) -> String {
    const MINUTE: i64 = 60;
    const HOUR: i64 = 60 * MINUTE;
    const DAY: i64 = 24 * HOUR;
    const WEEK: i64 = 7 * DAY;
    const MONTH: i64 = 30 * DAY;
    const YEAR: i64 = 365 * DAY;

    let elapsed = (now - created).max(0);
    let (count, unit) = match elapsed {
        e if e < MINUTE => return "just now".into(),
        e if e < HOUR => (e / MINUTE, "minute"),
        e if e < DAY => (e / HOUR, "hour"),
        e if e < WEEK => (e / DAY, "day"),
        e if e < MONTH => (e / WEEK, "week"),
        e if e < YEAR => (e / MONTH, "month"),
        e => (e / YEAR, "year"),
    };
    let plural = if count == 1 { "" } else { "s" };
    format!("{count} {unit}{plural} ago")
}

#[cfg(test)]
mod tests;
