// The expected values are byte ranges, not a vector of numbers.
#![allow(clippy::single_range_in_vec_init)]

use super::find_matches;

#[test]
fn finds_every_occurrence_ignoring_case_without_overlap() {
    let cases = [
        // An empty query matches with no ranges.
        ("anything", "", Some(vec![])),
        ("", "", Some(vec![])),
        ("Error: connect error", "ERROR", Some(vec![0..5, 15..20])),
        ("GET /healthz 200", "health", Some(vec![5..11])),
        ("aaaa", "aa", Some(vec![0..2, 2..4])),
        ("aaa", "aa", Some(vec![0..2])),
        ("GET / 200", "post", None),
        ("ab", "abc", None),
        ("", "a", None),
    ];
    for (text, query, ranges) in cases {
        assert_eq!(find_matches(text, query), ranges, "{query:?} in {text:?}");
    }
}

#[test]
fn ranges_are_byte_ranges_on_char_boundaries() {
    let text = "Größe ÜBER größe";
    let ranges = find_matches(text, "grö").expect("match");
    assert_eq!(ranges, vec![0..4, 14..18]);
    for range in ranges {
        assert!(text[range].eq_ignore_ascii_case("grö"));
    }
    assert_eq!(find_matches(text, "über"), Some(vec![8..13]));
}
