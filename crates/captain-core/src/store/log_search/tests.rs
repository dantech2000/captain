// The expected values are byte ranges, not a vector of numbers.
#![allow(clippy::single_range_in_vec_init)]

use super::find_matches;

#[test]
fn an_empty_query_matches_with_no_ranges() {
    assert_eq!(find_matches("anything", ""), Some(Vec::new()));
    assert_eq!(find_matches("", ""), Some(Vec::new()));
}

#[test]
fn finds_every_occurrence_ignoring_case() {
    assert_eq!(
        find_matches("Error: connect error", "ERROR"),
        Some(vec![0..5, 15..20])
    );
    assert_eq!(
        find_matches("GET /healthz 200", "health"),
        Some(vec![5..11])
    );
}

#[test]
fn a_missing_query_is_no_match() {
    assert_eq!(find_matches("GET / 200", "post"), None);
    assert_eq!(find_matches("ab", "abc"), None);
    assert_eq!(find_matches("", "a"), None);
}

#[test]
fn occurrences_do_not_overlap() {
    assert_eq!(find_matches("aaaa", "aa"), Some(vec![0..2, 2..4]));
    assert_eq!(find_matches("aaa", "aa"), Some(vec![0..2]));
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
