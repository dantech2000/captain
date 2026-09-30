use std::ops::Range;
use std::time::Duration;

use crate::search::fuzzy_match;

use super::action::{Action, Destination};
use super::catalog::{Catalog, Resolution, Target};
use super::duration::parse_duration;
use super::parse::{Scan, build, parse, scan};
use super::verb::{Flag, Verb};

/// The most suggestions for one line.
const MAX_SUGGESTIONS: usize = 8;
/// The times offered after `--since`.
const TIMES: [&str; 3] = ["10m", "1h", "1d"];

/// What a suggestion adds to the line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SuggestionKind {
    Verb(Verb),
    Target(Target),
    Flag(Flag),
    Time(Duration),
    /// The line as typed, which runs.
    Line,
}

/// One row of completions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suggestion {
    /// The command line the row stands for. Tab puts it in the search field.
    pub line: String,
    /// The typed parts of `line`, as byte ranges, to show in bold.
    pub bold: Vec<Range<usize>>,
    pub kind: SuggestionKind,
    /// What Enter runs. `None` when the line needs more words; Enter completes it.
    pub action: Option<Action>,
}

/// A row before ranking. The query matches `line` from byte `from` on.
struct Choice {
    line: String,
    from: usize,
    kind: SuggestionKind,
    action: Option<Action>,
}

/// Suggestions for `line`, best first: verbs while the first word is typed, then
/// the live names, options, and times the next word can take. A line whose first
/// word is not a verb gets none, so the palette searches instead.
pub fn complete(line: &str, catalog: &Catalog) -> Vec<Suggestion> {
    let words: Vec<(usize, &str)> = words(line);
    let Some(&(_, first)) = words.first() else {
        return Vec::new();
    };
    let trailing = line.ends_with(char::is_whitespace);
    if words.len() == 1 && !trailing {
        return rank(first, verbs(first, catalog), 0..0);
    }
    let Some(verb) = Verb::parse(first) else {
        return Vec::new();
    };
    let (done, partial, start) = match (trailing, words.split_last()) {
        (false, Some((&(start, partial), done))) => (&done[1..], partial, start),
        _ => (&words[1..], "", line.len()),
    };
    let prefix = &line[..start];
    let done: Vec<&str> = done.iter().map(|(_, word)| *word).collect();
    let Ok(scan) = scan(verb, &done) else {
        return Vec::new();
    };

    let options = if scan.wants_time {
        times(prefix, catalog)
    } else if partial.starts_with('-') || scan.names.len() >= verb.slots().len() {
        flags(verb, &scan, prefix, catalog)
    } else {
        names(verb, &scan, prefix, catalog)
    };
    let mut ranked = rank(partial, options, 0..prefix.trim_end().len());
    if let Ok(action) = parse(line, catalog) {
        let typed = line.trim_end();
        ranked.retain(|s| s.line != typed);
        ranked.insert(
            0,
            Suggestion {
                line: typed.into(),
                bold: std::iter::once(0..typed.len()).collect(),
                kind: SuggestionKind::Line,
                action: Some(action),
            },
        );
        ranked.truncate(MAX_SUGGESTIONS);
    }
    ranked
}

/// The words of `line` with their byte offsets.
fn words(line: &str) -> Vec<(usize, &str)> {
    line.split_whitespace()
        .map(|word| (word.as_ptr() as usize - line.as_ptr() as usize, word))
        .collect()
}

/// The verbs, and `go` with each page, that start like `word`.
fn verbs(word: &str, catalog: &Catalog) -> Vec<Choice> {
    let first = word.chars().next().map(|c| c.to_ascii_lowercase());
    let starts = |text: &str| text.chars().next() == first;
    let verbs = Verb::ALL
        .into_iter()
        .filter(|verb| *verb != Verb::Forward || catalog.kubernetes)
        .filter(|verb| starts(verb.name()))
        .map(|verb| Choice {
            line: verb.name().into(),
            from: 0,
            kind: SuggestionKind::Verb(verb),
            action: parse(verb.name(), catalog).ok(),
        });
    let pages = Destination::ALL
        .into_iter()
        .filter(|page| starts(page.name()))
        .map(|page| Choice {
            line: format!("go {}", page.name()),
            from: 3,
            kind: SuggestionKind::Target(Target::Page(page)),
            action: Some(Action::Go(page)),
        });
    verbs.chain(pages).collect()
}

/// The names the next word can take, each with the action it completes, if any.
fn names(verb: Verb, scan: &Scan, prefix: &str, catalog: &Catalog) -> Vec<Choice> {
    let slot = scan.names.len();
    let mut targets = Vec::new();
    for (name, kinds) in scan.names.iter().zip(verb.slots()) {
        match catalog.resolve(name, *kinds) {
            Resolution::Found(target) => targets.push(target),
            _ => return Vec::new(),
        }
    }
    catalog
        .candidates(verb.slots()[slot])
        .into_iter()
        .map(|(text, target)| {
            let mut words = Scan {
                names: scan.names.clone(),
                since: scan.since,
                errors: scan.errors,
                wants_time: false,
            };
            words.names.push(&text);
            let mut all = targets.clone();
            all.push(target.clone());
            let action = build(verb, &all, &words, catalog).ok();
            Choice {
                line: format!("{prefix}{text}"),
                from: prefix.len(),
                kind: SuggestionKind::Target(target),
                action,
            }
        })
        .collect()
}

/// The options the verb takes that the line does not have yet.
fn flags(verb: Verb, scan: &Scan, prefix: &str, catalog: &Catalog) -> Vec<Choice> {
    verb.flags()
        .iter()
        .filter(|flag| match flag {
            Flag::Since => scan.since.is_none(),
            Flag::Errors => !scan.errors,
        })
        .map(|flag| {
            let line = format!("{prefix}{}", flag.text());
            Choice {
                action: parse(&line, catalog).ok(),
                line,
                from: prefix.len(),
                kind: SuggestionKind::Flag(*flag),
            }
        })
        .collect()
}

fn times(prefix: &str, catalog: &Catalog) -> Vec<Choice> {
    TIMES
        .into_iter()
        .filter_map(|time| {
            let line = format!("{prefix}{time}");
            Some(Choice {
                action: parse(&line, catalog).ok(),
                line,
                from: prefix.len(),
                kind: SuggestionKind::Time(parse_duration(time).ok()?),
            })
        })
        .collect()
}

/// Keeps the options that match `partial`, best first; ties keep their order. The
/// `typed` range and the matched characters are bold.
fn rank(partial: &str, options: Vec<Choice>, typed: Range<usize>) -> Vec<Suggestion> {
    let mut scored: Vec<(i32, Suggestion)> = options
        .into_iter()
        .filter_map(|option| {
            let found = fuzzy_match(partial, &option.line[option.from..])?;
            let mut bold: Vec<Range<usize>> = Vec::new();
            if !typed.is_empty() {
                bold.push(typed.clone());
            }
            bold.extend(
                found
                    .ranges
                    .into_iter()
                    .map(|r| r.start + option.from..r.end + option.from),
            );
            Some((
                found.score,
                Suggestion {
                    line: option.line,
                    bold,
                    kind: option.kind,
                    action: option.action,
                },
            ))
        })
        .collect();
    scored.sort_by_key(|(score, _)| -score);
    scored.truncate(MAX_SUGGESTIONS);
    scored
        .into_iter()
        .map(|(_, suggestion)| suggestion)
        .collect()
}

#[cfg(test)]
mod tests;
