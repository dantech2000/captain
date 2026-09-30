//! Masks secret-looking values in text that agents read: log lines and
//! environment values. It errs toward masking; a masked value that was not a secret
//! costs less than a leaked one.

use std::ops::Range;

use crate::model::is_secret_key;

/// What a masked value becomes.
pub const MASK: &str = "[masked]";

/// Quotes and brackets around a value. They stay, so the text keeps its shape.
const WRAPPERS: &[char] = &[
    '"', '\'', '`', ',', ';', '(', ')', '[', ']', '{', '}', '<', '>',
];

/// Prefixes of well-known API tokens: GitHub, GitLab, Slack, Stripe, npm, and
/// Anthropic.
const TOKEN_PREFIXES: &[&str] = &[
    "ghp_",
    "gho_",
    "ghu_",
    "ghs_",
    "ghr_",
    "github_pat_",
    "glpat-",
    "xoxb-",
    "xoxp-",
    "xoxa-",
    "xoxs-",
    "sk_live_",
    "rk_live_",
    "npm_",
    "sk-ant-",
];

/// `line` with secret-looking values replaced by [`MASK`]: the value after every
/// secret-looking key (`PASSWORD=…`, `"token":"…"`, `--api-key=…`, a whole quoted
/// value such as `password="a b c"`), the token after `Bearer` or `Basic`, the
/// password in a URL (`postgres://user:…@host`), and well-known token shapes
/// (GitHub, Slack, AWS access keys, JWTs). Quotes and brackets around a value stay.
pub fn mask_secrets(line: &str) -> String {
    let mut spans = Vec::new();
    assigned_values(line, &mut spans);
    scheme_tokens(line, &mut spans);
    url_passwords(line, &mut spans);
    known_tokens(line, &mut spans);
    replace_spans(line, spans)
}

/// The values after `key=`, `key:`, `key =`, `"key":`, and `key =>` when the key
/// looks secret.
fn assigned_values(line: &str, spans: &mut Vec<Range<usize>>) {
    let bytes = line.as_bytes();
    for (at, &byte) in bytes.iter().enumerate() {
        if !(byte == b'=' || byte == b':' && !line[at + 1..].starts_with("//")) {
            continue;
        }
        if !is_secret_key(&bare_key(key_before(line, at))) {
            continue;
        }
        let mut start = at + 1;
        if byte == b'=' && bytes.get(start) == Some(&b'>') {
            start += 1;
        }
        while bytes.get(start).is_some_and(|b| *b == b' ' || *b == b'\t') {
            start += 1;
        }
        spans.extend(value_at(line, start));
    }
}

/// The key that ends just before the separator at `at`, with its quotes: `"token"`
/// in `"token": "…"`, or `DB_PASSWORD` in `DB_PASSWORD=…`.
fn key_before(line: &str, at: usize) -> &str {
    let bytes = line.as_bytes();
    let mut end = at;
    while end > 0 && bytes[end - 1] == b' ' {
        end -= 1;
    }
    let mut start = end;
    while start > 0 && (is_quote(bytes[start - 1]) || bytes[start - 1] == b'\\') {
        start -= 1;
    }
    let word_end = start;
    while start > 0 && is_word(bytes[start - 1]) {
        start -= 1;
    }
    if start == word_end {
        return "";
    }
    &line[start..word_end]
}

/// A key without its leading dashes: `--api-key` gives `api_key`. The secret
/// markers use `_`, so `-` counts as `_`.
fn bare_key(key: &str) -> String {
    key.trim_start_matches('-').replace('-', "_")
}

/// The value that starts at `start`: everything inside its quotes when it is
/// quoted (a JSON string, or `\"…\"` inside one), else the word up to a space or
/// quote, without the brackets and punctuation around it.
fn value_at(line: &str, start: usize) -> Option<Range<usize>> {
    let bytes = line.as_bytes();
    let rest = &line[start..];
    let span = if rest.starts_with("\\\"") {
        let from = start + 2;
        let end = line[from..].find("\\\"").map_or(line.len(), |at| from + at);
        from..end
    } else if let Some(&quote) = bytes.get(start).filter(|b| is_quote(**b)) {
        let from = start + 1;
        from..closing_quote(bytes, from, quote)
    } else {
        let end = rest
            .find(|c: char| c.is_whitespace() || c.is_ascii() && is_quote(c as u8))
            .map_or(line.len(), |at| start + at);
        let word = &line[start..end];
        let lead = word.len() - word.trim_start_matches(WRAPPERS).len();
        let tail = word.trim_end_matches(WRAPPERS).len().max(lead);
        start + lead..start + tail
    };
    (!span.is_empty()).then_some(span)
}

/// The index of the `quote` that closes a value starting at `from`, or the end of
/// the line. In `"…"`, a backslash escapes the next byte.
fn closing_quote(bytes: &[u8], from: usize, quote: u8) -> usize {
    let mut at = from;
    while at < bytes.len() {
        match bytes[at] {
            b'\\' if quote == b'"' => at += 2,
            byte if byte == quote => return at,
            _ => at += 1,
        }
    }
    bytes.len()
}

/// The token after `Bearer` or `Basic`.
fn scheme_tokens(line: &str, spans: &mut Vec<Range<usize>>) {
    let bytes = line.as_bytes();
    for word in words(line) {
        let scheme = &line[word.clone()];
        if !(scheme.eq_ignore_ascii_case("bearer") || scheme.eq_ignore_ascii_case("basic")) {
            continue;
        }
        let mut start = word.end;
        if !bytes.get(start).is_some_and(u8::is_ascii_whitespace) {
            continue;
        }
        while bytes.get(start).is_some_and(u8::is_ascii_whitespace) {
            start += 1;
        }
        spans.extend(value_at(line, start));
    }
}

/// The password of each URL: `redis://:hunter2@cache:6379` gives
/// `redis://:[masked]@cache:6379`.
fn url_passwords(line: &str, spans: &mut Vec<Range<usize>>) {
    for (at, _) in line.match_indices("://") {
        let start = at + 3;
        let rest = &line[start..];
        let end = rest
            .find(|c: char| matches!(c, '/' | '?' | '#' | '"' | '\'' | '`') || c.is_whitespace())
            .unwrap_or(rest.len());
        let Some(user_end) = rest[..end].rfind('@') else {
            continue;
        };
        if let Some(colon) = rest[..user_end].find(':') {
            spans.push(start + colon + 1..start + user_end);
        }
    }
}

/// Words that have the shape of a well-known token.
fn known_tokens(line: &str, spans: &mut Vec<Range<usize>>) {
    for word in words(line) {
        let text = line[word.clone()].trim_end_matches(['.', '-']);
        if looks_like_token(text) {
            spans.push(word.start..word.start + text.len());
        }
    }
}

/// The runs of word bytes in `line`: letters, digits, `_`, `-`, and `.`.
fn words(line: &str) -> impl Iterator<Item = Range<usize>> + '_ {
    let bytes = line.as_bytes();
    let mut at = 0;
    std::iter::from_fn(move || {
        while at < bytes.len() && !is_word(bytes[at]) {
            at += 1;
        }
        let start = at;
        while at < bytes.len() && is_word(bytes[at]) {
            at += 1;
        }
        (start < at).then_some(start..at)
    })
}

fn is_word(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.')
}

fn is_quote(byte: u8) -> bool {
    matches!(byte, b'"' | b'\'' | b'`')
}

/// `line` with each span, merged where they overlap, replaced by [`MASK`].
fn replace_spans(line: &str, mut spans: Vec<Range<usize>>) -> String {
    spans.sort_by_key(|span| span.start);
    let mut out = String::with_capacity(line.len());
    let mut done = 0;
    for span in spans {
        if span.end <= done {
            continue;
        }
        if span.start >= done {
            out.push_str(&line[done..span.start]);
            out.push_str(MASK);
        }
        done = span.end;
    }
    out.push_str(&line[done..]);
    out
}

fn looks_like_token(word: &str) -> bool {
    let prefixed = TOKEN_PREFIXES.iter().any(|p| word.starts_with(p)) && word.len() >= 16;
    let openai = word.starts_with("sk-") && word.len() >= 24;
    let aws = word.len() == 20
        && word.starts_with("AKIA")
        && word
            .bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit());
    let jwt = word.starts_with("eyJ") && word.len() >= 30 && word.split('.').count() == 3;
    prefixed || openai || aws || jwt
}

#[cfg(test)]
mod tests;
