use super::age_label;

const NOW: i64 = 1_800_000_000;

#[test]
fn ages_use_the_largest_whole_unit() {
    let cases = [
        (NOW - 59, "just now"),
        // A clock skew must not show a future age.
        (NOW + 100, "just now"),
        (NOW - 60, "1 minute ago"),
        (NOW - 3 * 3600, "3 hours ago"),
        (NOW - 2 * 86_400, "2 days ago"),
        (NOW - 14 * 86_400, "2 weeks ago"),
        (NOW - 400 * 86_400, "1 year ago"),
        // Docker reports a missing time as 0.
        (0, "—"),
    ];
    for (time, label) in cases {
        assert_eq!(age_label(time, NOW), label, "{time}");
    }
}
