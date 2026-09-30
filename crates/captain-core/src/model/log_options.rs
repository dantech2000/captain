/// Which lines [`ContainerApi::logs_with`](crate::ContainerApi::logs_with) sends,
/// and whether it then follows new ones.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LogOptions {
    /// At most this many of the past lines. `None` sends all of them.
    pub tail: Option<usize>,
    /// Only lines from this time on, in Unix seconds.
    pub since: Option<i64>,
    /// Keep the stream open for new lines. Without it, the stream ends after the
    /// past lines.
    pub follow: bool,
}
