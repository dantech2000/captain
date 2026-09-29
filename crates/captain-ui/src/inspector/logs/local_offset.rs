use chrono::Local;

/// Seconds east of UTC for this machine's time zone, right now. Read on each call, so
/// a daylight saving change during a session shows at once.
pub fn local_offset() -> i32 {
    Local::now().offset().local_minus_utc()
}
