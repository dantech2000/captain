//! Masks secret-looking values in text that agents read: log lines and
//! environment values. It errs toward masking; a masked value that was not a secret
//! costs less than a leaked one.

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

/// `line` with secret-looking values replaced by [`MASK`]: the value after a
/// secret-looking key (`PASSWORD=…`, `"token": "…"`, `--api-key=…`), the token
/// after `Bearer` or `Basic`, the password in a URL (`postgres://user:…@host`), and
/// well-known token shapes (GitHub, Slack, AWS access keys, JWTs).
pub fn mask_secrets(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut mask_next = false;
    for piece in line.split_inclusive(char::is_whitespace) {
        let word = piece.trim_end_matches(char::is_whitespace);
        let space = &piece[word.len()..];
        if word.is_empty() {
            out.push_str(piece);
            continue;
        }
        if mask_next {
            out.push_str(&mask_value(word));
            mask_next = false;
        } else {
            out.push_str(&mask_word(word));
            mask_next = opens_secret(word);
        }
        out.push_str(space);
    }
    out
}

/// One word with no spaces: a `key=value` pair with a secret key, a URL with a
/// password, or a token.
fn mask_word(word: &str) -> String {
    if let Some((key, separator, value)) = split_pair(word)
        && is_secret_key(&bare_key(key))
        && !value.is_empty()
    {
        return format!("{key}{separator}{}", mask_value(value));
    }
    if let Some(masked) = mask_url_password(word) {
        return masked;
    }
    if looks_like_token(core(word)) {
        return mask_value(word);
    }
    word.to_string()
}

/// Splits `word` at the first `=`, or at a `:` that does not start `://`.
fn split_pair(word: &str) -> Option<(&str, char, &str)> {
    let at = word.find('=').or_else(|| {
        word.match_indices(':')
            .map(|(at, _)| at)
            .find(|at| !word[at + 1..].starts_with("//"))
    })?;
    let separator = word[at..].chars().next()?;
    Some((&word[..at], separator, &word[at + 1..]))
}

/// A key without its quotes and leading dashes: `"--api-key"` gives `api-key`. The
/// secret markers use `_`, so `-` counts as `_`.
fn bare_key(key: &str) -> String {
    key.trim_matches(WRAPPERS)
        .trim_start_matches('-')
        .replace('-', "_")
}

/// True if the word leaves its value to the next word: `password:` or `"token":`,
/// or an auth scheme such as `Bearer`.
fn opens_secret(word: &str) -> bool {
    let bare = word.trim_matches(WRAPPERS);
    if bare.eq_ignore_ascii_case("bearer") || bare.eq_ignore_ascii_case("basic") {
        return true;
    }
    word.strip_suffix([':', '='])
        .is_some_and(|key| is_secret_key(&bare_key(key)))
}

/// The password of a URL in `word`, masked: `redis://:hunter2@cache:6379` gives
/// `redis://:[masked]@cache:6379`.
fn mask_url_password(word: &str) -> Option<String> {
    let start = word.find("://")? + 3;
    let rest = &word[start..];
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let at = rest[..end].rfind('@')?;
    let colon = rest[..at].find(':')?;
    Some(format!(
        "{}{}:{MASK}{}",
        &word[..start],
        &rest[..colon],
        &rest[at..]
    ))
}

/// The word without the quotes and brackets around it.
fn core(word: &str) -> &str {
    word.trim_matches(WRAPPERS)
}

/// `value` with its core replaced by [`MASK`], keeping quotes and brackets.
fn mask_value(value: &str) -> String {
    let lead = value.len() - value.trim_start_matches(WRAPPERS).len();
    let tail = value.trim_end_matches(WRAPPERS).len().max(lead);
    if lead == tail {
        return value.to_string();
    }
    format!("{}{MASK}{}", &value[..lead], &value[tail..])
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
