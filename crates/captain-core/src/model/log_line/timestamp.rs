//! The RFC 3339 time that Docker puts in front of each line when a log request asks
//! for timestamps, for example `2024-05-01T12:34:56.123456789Z`.

const DAY: i64 = 24 * 60 * 60;

/// Splits `raw` into its leading time, as Unix seconds, and the text after the space.
/// Returns `None` when `raw` does not start with a time.
pub(super) fn split(raw: &str) -> Option<(i64, &str)> {
    match raw.split_once(' ') {
        Some((head, text)) => parse(head).map(|time| (time, text)),
        None => parse(raw).map(|time| (time, "")),
    }
}

/// Reads `YYYY-MM-DDTHH:MM:SS`, an optional fraction, then `Z` or `±HH:MM`, as Unix
/// seconds. The fraction is dropped.
pub fn parse(s: &str) -> Option<i64> {
    let b = s.as_bytes();
    if b.len() < 20 || b[4] != b'-' || b[7] != b'-' || b[10] != b'T' {
        return None;
    }
    if b[13] != b':' || b[16] != b':' {
        return None;
    }
    let year = number(&s[0..4])?;
    let month = number(&s[5..7])?;
    let day = number(&s[8..10])?;
    let hour = number(&s[11..13])?;
    let minute = number(&s[14..16])?;
    let second = number(&s[17..19])?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    if hour > 23 || minute > 59 || second > 60 {
        return None;
    }

    let mut rest = &s[19..];
    if let Some(fraction) = rest.strip_prefix('.') {
        let digits = fraction.bytes().take_while(u8::is_ascii_digit).count();
        if digits == 0 {
            return None;
        }
        rest = &fraction[digits..];
    }
    let offset = zone_offset(rest)?;

    let days = days_from_civil(year, month, day);
    Some(days * DAY + hour * 3600 + minute * 60 + second - offset)
}

/// The fraction of the second in the time at the start of `raw`, in nanoseconds.
/// Digits past the ninth are dropped; no fraction reads as 0.
pub(super) fn nanos(raw: &str) -> u32 {
    let Some(fraction) = raw.get(19..).and_then(|rest| rest.strip_prefix('.')) else {
        return 0;
    };
    let digits: String = fraction
        .bytes()
        .take_while(u8::is_ascii_digit)
        .take(9)
        .map(char::from)
        .collect();
    format!("{digits:0<9}").parse().unwrap_or(0)
}

/// Formats Unix seconds as `HH:MM:SS`, shifted by `offset` seconds east of UTC.
pub(super) fn clock(time: i64, offset: i32) -> String {
    let t = (time + i64::from(offset)).rem_euclid(DAY);
    format!("{:02}:{:02}:{:02}", t / 3600, t / 60 % 60, t % 60)
}

/// Seconds east of UTC for `Z` or `±HH:MM`. Anything else is not a zone.
fn zone_offset(zone: &str) -> Option<i64> {
    if zone == "Z" || zone == "z" {
        return Some(0);
    }
    let b = zone.as_bytes();
    if b.len() != 6 || b[3] != b':' {
        return None;
    }
    let sign = match b[0] {
        b'+' => 1,
        b'-' => -1,
        _ => return None,
    };
    let hours = number(&zone[1..3])?;
    let minutes = number(&zone[4..6])?;
    Some(sign * (hours * 3600 + minutes * 60))
}

/// Parses a run of ASCII digits. Signs and spaces are not digits.
fn number(digits: &str) -> Option<i64> {
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

/// Days since 1970-01-01 for a date in the proleptic Gregorian calendar. This is
/// Howard Hinnant's `days_from_civil`.
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let month_from_march = (month + 9) % 12;
    let day_of_year = (153 * month_from_march + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

#[cfg(test)]
mod tests;
