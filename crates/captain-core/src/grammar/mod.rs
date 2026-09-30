//! The command grammar of the ⌘K palette: `restart api`, `logs worker --since 10m`,
//! `forward svc/web 8080`. It parses a line against live names and completes it.
//! See docs/features/0033-command-grammar.md.

mod action;
mod catalog;
mod complete;
mod duration;
mod examples;
mod parse;
mod verb;

pub use action::{Action, Destination};
pub use catalog::{Catalog, Target};
pub use complete::{Suggestion, SuggestionKind, complete};
pub use duration::{duration_label, parse_duration};
pub use examples::examples;
pub use parse::{ParseError, parse};
pub use verb::{Flag, Verb};

#[cfg(test)]
mod fixture;
