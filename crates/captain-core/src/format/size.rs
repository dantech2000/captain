const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];

/// A byte count in binary units with at most one decimal, for example `182 MB` or `1.2 GB`.
pub fn bytes_label(bytes: u64) -> String {
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 || value >= 100.0 {
        format!("{value:.0} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

/// A transfer rate, for example `8.1 KB/s`.
pub fn rate_label(bytes_per_second: u64) -> String {
    format!("{}/s", bytes_label(bytes_per_second))
}

/// A percent with one decimal below 10 and none above, for example `3.1%` or `42%`.
pub fn percent_label(percent: f64) -> String {
    // Rounding can produce -0.0, and CPU counters can step backwards by a hair.
    let percent = percent.max(0.0);
    if percent < 10.0 {
        format!("{percent:.1}%")
    } else {
        format!("{percent:.0}%")
    }
}

/// A byte count for a tight line: one decimal below 10, else none, for example
/// `55 MB` or `1.6 GB`.
pub fn short_bytes_label(bytes: u64) -> String {
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 || value >= 10.0 {
        format!("{value:.0} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

/// A whole percent for a tight line, for example `0%` or `42%`.
pub fn short_percent_label(percent: f64) -> String {
    format!("{:.0}%", percent.max(0.0))
}

#[cfg(test)]
mod tests;
