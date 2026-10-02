// The expected values are byte ranges, not a vector of numbers.
#![allow(clippy::single_range_in_vec_init)]

use super::fuzzy_match;

fn score(query: &str, candidate: &str) -> i32 {
    fuzzy_match(query, candidate)
        .unwrap_or_else(|| panic!("{query:?} should match {candidate:?}"))
        .score
}

#[test]
fn empty_query_matches_everything() {
    let found = fuzzy_match("", "Restart api").unwrap();
    assert_eq!(found.score, 0);
    assert!(found.ranges.is_empty());
}

#[test]
fn rejects_a_query_that_is_not_a_subsequence() {
    assert_eq!(fuzzy_match("xyz", "Restart api"), None);
    assert_eq!(fuzzy_match("tr", "rt"), None);
    assert_eq!(fuzzy_match("longer", "long"), None);
}

#[test]
fn ranges_cover_the_matched_bytes() {
    let cases = [
        // Case does not matter, and adjacent characters join into one range.
        ("REST", "restart", vec![0..4]),
        ("rest", "Restart api", vec![0..4]),
        // Whitespace in the query is ignored.
        ("go con", "Go to Containers", vec![0..2, 6..9]),
        // Camel-case humps count as word starts.
        ("cv", "ContainersView", vec![0..1, 10..11]),
        // A greedy match takes the first "a"; the word "api" scores higher.
        ("ap", "a api", vec![2..4]),
        // Ranges are in bytes for multibyte characters.
        ("fé", "Café", vec![2..5]),
        ("É", "café", vec![3..5]),
    ];
    for (query, candidate, ranges) in cases {
        let found = fuzzy_match(query, candidate).unwrap();
        assert_eq!(found.ranges, ranges, "{query:?} in {candidate:?}");
    }
}

#[test]
fn prefers_prefixes_word_starts_runs_and_fewer_skips() {
    let cases = [
        ("res", "Restart", "Unrestricted"),
        ("api", "Restart api", "rapid"),
        ("abc", "abcxx", "axbxc"),
        ("ab", "axb", "axxxxb"),
    ];
    for (query, better, worse) in cases {
        assert!(
            score(query, better) > score(query, worse),
            "{query:?}: {better:?} should beat {worse:?}"
        );
    }
}
