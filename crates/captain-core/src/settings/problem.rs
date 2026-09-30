use std::fmt;

/// The reference page that every problem links to.
pub const REFERENCE_URL: &str =
    "https://github.com/dantech2000/captain/blob/main/docs/reference/settings.md";

/// A mistake in `settings.json`: bad JSON, or a value that a key does not take.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileProblem {
    /// The line of the mistake, from 1.
    pub line: usize,
    /// The key with the bad value, such as `kubernetes.port`. `None` for bad JSON.
    pub key: Option<String>,
    /// What is wrong, such as "must be true or false".
    pub message: String,
}

impl FileProblem {
    /// The reference section of the key, or the top of the reference.
    pub fn link(&self) -> String {
        match &self.key {
            Some(key) => doc_link(key),
            None => REFERENCE_URL.to_string(),
        }
    }
}

/// "settings.json line 9: kubernetes.port must be … See <link>."
impl fmt::Display for FileProblem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "settings.json line {}: ", self.line)?;
        if let Some(key) = &self.key {
            write!(f, "{key} ")?;
        }
        write!(f, "{}. See {}", self.message, self.link())
    }
}

/// The link to `key` in docs/reference/settings.md, such as
/// `…/settings.md#kubernetesport`.
pub fn doc_link(key: &str) -> String {
    format!("{REFERENCE_URL}#{}", anchor(key))
}

/// GitHub's anchor for the heading `` `key` ``: lowercase, with only letters,
/// digits, `-`, and `_` kept.
pub(super) fn anchor(key: &str) -> String {
    key.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .map(|c| c.to_ascii_lowercase())
        .collect()
}
