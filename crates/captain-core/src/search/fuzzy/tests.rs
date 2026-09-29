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
fn ignores_case() {
    let found = fuzzy_match("REST", "restart").unwrap();
    assert_eq!(found.ranges, vec![0..4]);
}

#[test]
fn joins_adjacent_characters_into_one_range() {
    let found = fuzzy_match("rest", "Restart api").unwrap();
    assert_eq!(found.ranges, vec![0..4]);
}

#[test]
fn ignores_whitespace_in_the_query() {
    let found = fuzzy_match("go con", "Go to Containers").unwrap();
    assert_eq!(found.ranges, vec![0..2, 6..9]);
}

#[test]
fn prefers_a_prefix() {
    assert!(score("res", "Restart") > score("res", "Unrestricted"));
}

#[test]
fn prefers_word_starts() {
    assert!(score("api", "Restart api") > score("api", "rapid"));
}

#[test]
fn prefers_camel_case_word_starts() {
    let found = fuzzy_match("cv", "ContainersView").unwrap();
    assert_eq!(found.ranges, vec![0..1, 10..11]);
}

#[test]
fn prefers_contiguous_runs() {
    assert!(score("abc", "abcxx") > score("abc", "axbxc"));
}

#[test]
fn prefers_fewer_skipped_characters() {
    assert!(score("ab", "axb") > score("ab", "axxxxb"));
}

#[test]
fn picks_the_best_alignment_not_the_first() {
    // A greedy match takes the first "a"; the word "api" scores higher.
    let found = fuzzy_match("ap", "a api").unwrap();
    assert_eq!(found.ranges, vec![2..4]);
}

#[test]
fn reports_byte_ranges_for_multibyte_characters() {
    let found = fuzzy_match("fé", "Café").unwrap();
    assert_eq!(found.ranges, vec![2..5]);
    let found = fuzzy_match("É", "café").unwrap();
    assert_eq!(found.ranges, vec![3..5]);
}
