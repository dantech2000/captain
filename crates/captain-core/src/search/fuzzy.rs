use std::ops::Range;

/// Points for each matched character.
const MATCH: i32 = 1;
/// Extra points when the first character of the candidate matches.
const PREFIX: i32 = 12;
/// Extra points for a match at the start of a word.
const WORD_START: i32 = 8;
/// Extra points for a match right after the previous match.
const CONSECUTIVE: i32 = 6;
/// Points lost for each skipped character between two matches.
const GAP: i32 = 1;

/// No path reaches this cell.
const NONE: i32 = i32::MIN;

/// How well a query matches a candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuzzyMatch {
    /// Higher is better. Compare scores only between matches of the same query.
    pub score: i32,
    /// The matched characters as byte ranges into the candidate, in order.
    /// Adjacent matched characters share one range.
    pub ranges: Vec<Range<usize>>,
}

/// Matches `query` as a case-insensitive subsequence of `candidate`. Whitespace in
/// the query is ignored. Prefix matches, word starts, and runs of adjacent characters
/// score higher; skipped characters between matches score lower. An empty query
/// matches everything with a score of zero.
pub fn fuzzy_match(query: &str, candidate: &str) -> Option<FuzzyMatch> {
    let query: Vec<char> = query
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(fold)
        .collect();
    if query.is_empty() {
        return Some(FuzzyMatch {
            score: 0,
            ranges: Vec::new(),
        });
    }
    let chars: Vec<(usize, char)> = candidate.char_indices().collect();
    if query.len() > chars.len() {
        return None;
    }

    let (best, from) = score_table(&query, &chars);
    let last = &best[query.len() - 1];
    let mut end = None;
    for (j, &score) in last.iter().enumerate() {
        if score != NONE && end.is_none_or(|e: usize| score > last[e]) {
            end = Some(j);
        }
    }
    let end = end?;

    let mut positions = vec![end; query.len()];
    for i in (1..query.len()).rev() {
        positions[i - 1] = from[i][positions[i]];
    }
    Some(FuzzyMatch {
        score: last[end],
        ranges: ranges(&chars, &positions),
    })
}

/// `best[i][j]` is the best score with `query[..=i]` matched and `query[i]` at
/// `chars[j]`. `from[i][j]` is where `query[i - 1]` matched on that path.
fn score_table(query: &[char], chars: &[(usize, char)]) -> (Vec<Vec<i32>>, Vec<Vec<usize>>) {
    let (m, n) = (query.len(), chars.len());
    let mut best = vec![vec![NONE; n]; m];
    let mut from = vec![vec![0; n]; m];
    for i in 0..m {
        // The best `best[i - 1][k] + GAP * k` over every `k < j - 1`, and its `k`.
        // A gap from `k` to `j` then costs `GAP * (j - k - 1)`.
        let mut gap_best: Option<(i32, usize)> = None;
        for j in i..n {
            if i > 0 && j >= 2 && best[i - 1][j - 2] != NONE {
                let k = j - 2;
                let value = best[i - 1][k] + GAP * k as i32;
                if gap_best.is_none_or(|(b, _)| value > b) {
                    gap_best = Some((value, k));
                }
            }
            if fold(chars[j].1) != query[i] {
                continue;
            }
            let bonus = MATCH + position_bonus(chars, j);
            if i == 0 {
                best[0][j] = bonus;
                continue;
            }
            let mut score = NONE;
            let mut prev = 0;
            if best[i - 1][j - 1] != NONE {
                score = best[i - 1][j - 1] + CONSECUTIVE;
                prev = j - 1;
            }
            if let Some((value, k)) = gap_best {
                let value = value - GAP * (j as i32 - 1);
                if value > score {
                    score = value;
                    prev = k;
                }
            }
            if score != NONE {
                best[i][j] = score + bonus;
                from[i][j] = prev;
            }
        }
    }
    (best, from)
}

/// Extra points for where `chars[j]` sits: the first character, or a word start
/// after a separator or a lowercase-to-uppercase change.
fn position_bonus(chars: &[(usize, char)], j: usize) -> i32 {
    if j == 0 {
        return PREFIX;
    }
    let prev = chars[j - 1].1;
    let current = chars[j].1;
    if !prev.is_alphanumeric() || (prev.is_lowercase() && current.is_uppercase()) {
        WORD_START
    } else {
        0
    }
}

/// Byte ranges for the matched character positions, with adjacent ones joined.
fn ranges(chars: &[(usize, char)], positions: &[usize]) -> Vec<Range<usize>> {
    let mut ranges: Vec<Range<usize>> = Vec::new();
    for &p in positions {
        let (start, c) = chars[p];
        let end = start + c.len_utf8();
        match ranges.last_mut() {
            Some(last) if last.end == start => last.end = end,
            _ => ranges.push(start..end),
        }
    }
    ranges
}

/// Lowercases one character. Characters that lowercase to several keep their first.
fn fold(c: char) -> char {
    c.to_lowercase().next().unwrap_or(c)
}

#[cfg(test)]
mod tests;
