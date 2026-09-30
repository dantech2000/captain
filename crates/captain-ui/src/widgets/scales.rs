//! The sparkline scales each kind of metric uses, so every chart of a metric matches.

use super::Scale;

/// CPU in percent. Anything under 1% draws flat.
pub const CPU: Scale = Scale::FromZero { floor: 5.0 };
/// Memory in bytes. Small changes in a large value stay visible.
pub const MEMORY: Scale = Scale::Range;
/// Network in bytes per second. Anything under 1 KB/s draws flat.
pub const NETWORK: Scale = Scale::FromZero { floor: 1024.0 };
/// Container counts.
pub const COUNT: Scale = Scale::FromZero { floor: 1.0 };
