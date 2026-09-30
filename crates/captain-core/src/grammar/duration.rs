use std::time::Duration;

/// Parses a time such as `30s`, `10m`, `1h`, or `2d`: a whole number and one unit.
pub fn parse_duration(text: &str) -> Result<Duration, String> {
    let invalid = || {
        format!(
            "\u{201c}{text}\u{201d} is not a time. Use a number and s, m, h, or d, such as 10m."
        )
    };
    let split = text
        .find(|c: char| !c.is_ascii_digit())
        .ok_or_else(invalid)?;
    let (number, unit) = text.split_at(split);
    let number: u64 = number.parse().map_err(|_| invalid())?;
    let seconds = match unit {
        "s" => 1,
        "m" => 60,
        "h" => 60 * 60,
        "d" => 24 * 60 * 60,
        _ => return Err(invalid()),
    };
    if number == 0 {
        return Err("Use a time above zero, such as 10m.".into());
    }
    number
        .checked_mul(seconds)
        .map(Duration::from_secs)
        .ok_or_else(invalid)
}

/// A time for a row, for example "the last 10 minutes" for 10m.
pub fn duration_label(duration: Duration) -> String {
    let seconds = duration.as_secs();
    let (count, unit) = [(86_400, "day"), (3_600, "hour"), (60, "minute")]
        .into_iter()
        .find(|(size, _)| seconds.is_multiple_of(*size))
        .map(|(size, unit)| (seconds / size, unit))
        .unwrap_or((seconds, "second"));
    match count {
        1 => format!("the last {unit}"),
        n => format!("the last {n} {unit}s"),
    }
}

#[cfg(test)]
mod tests;
