use super::age_label;

const NOW: i64 = 1_800_000_000;

#[test]
fn under_a_minute_is_just_now() {
    assert_eq!(age_label(NOW - 59, NOW), "just now");
}

#[test]
fn future_timestamps_are_just_now() {
    assert_eq!(age_label(NOW + 100, NOW), "just now");
}

#[test]
fn picks_the_largest_whole_unit() {
    assert_eq!(age_label(NOW - 60, NOW), "1 minute ago");
    assert_eq!(age_label(NOW - 3 * 3600, NOW), "3 hours ago");
    assert_eq!(age_label(NOW - 2 * 86_400, NOW), "2 days ago");
    assert_eq!(age_label(NOW - 14 * 86_400, NOW), "2 weeks ago");
    assert_eq!(age_label(NOW - 400 * 86_400, NOW), "1 year ago");
}

#[test]
fn missing_time_shows_a_dash() {
    assert_eq!(age_label(0, NOW), "—");
}
