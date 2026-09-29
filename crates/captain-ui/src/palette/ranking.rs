use std::ops::Range;

use captain_core::search::fuzzy_match;

use super::command::{Command, Section};

/// The most rows the palette lists for a query.
const MAX_RESULTS: usize = 50;

/// A command that matches the query, with the matched ranges of its title.
#[derive(Debug, Clone)]
pub struct Ranked {
    pub command: Command,
    pub ranges: Vec<Range<usize>>,
}

/// The commands to list for `query`, in display order. Rows of one section are
/// adjacent. An empty query lists the suggested commands in their sections.
/// Otherwise the best matches come first, and a section is placed by its best match.
pub fn rank(commands: Vec<Command>, query: &str) -> Vec<Ranked> {
    if query.trim().is_empty() {
        let mut suggested: Vec<Ranked> = commands
            .into_iter()
            .filter(|c| c.suggested)
            .map(|command| Ranked {
                command,
                ranges: Vec::new(),
            })
            .collect();
        suggested.sort_by_key(|r| r.command.section as usize);
        return suggested;
    }

    let mut scored: Vec<(i32, Ranked)> = commands
        .into_iter()
        .filter_map(|command| {
            let found = fuzzy_match(query, &command.title)?;
            Some((
                found.score,
                Ranked {
                    command,
                    ranges: found.ranges,
                },
            ))
        })
        .collect();
    scored.sort_by_key(|(score, _)| -score);
    scored.truncate(MAX_RESULTS);

    let mut order: Vec<Section> = Vec::new();
    for (_, ranked) in &scored {
        if !order.contains(&ranked.command.section) {
            order.push(ranked.command.section);
        }
    }
    let mut ranked: Vec<Ranked> = scored.into_iter().map(|(_, r)| r).collect();
    ranked.sort_by_key(|r| order.iter().position(|s| *s == r.command.section));
    ranked
}

/// The index of row `row` among the list's children, which put a header before
/// the first row of each section.
pub fn child_index(results: &[Ranked], row: usize) -> usize {
    let headers = (0..=row.min(results.len().saturating_sub(1)))
        .filter(|&i| i == 0 || results[i].command.section != results[i - 1].command.section)
        .count();
    row + headers
}

#[cfg(test)]
mod tests;
