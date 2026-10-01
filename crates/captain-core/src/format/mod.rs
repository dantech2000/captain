//! Text formatting for values that views show.

mod age;
mod size;

pub use age::age_label;
pub use size::{bytes_label, percent_label, rate_label, short_bytes_label, short_percent_label};
