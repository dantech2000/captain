use std::fmt;

/// An image reference to pull, split the way the Engine API wants it.
/// `busybox` becomes `busybox` with tag `latest`, so a pull never fetches every tag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageReference {
    /// The repository, or the repository with a digest (`app@sha256:…`).
    pub name: String,
    /// The tag. Empty when the reference has a digest.
    pub tag: String,
}

impl ImageReference {
    /// Parses user input. Returns `None` for empty input or input with spaces.
    pub fn parse(input: &str) -> Option<Self> {
        let input = input.trim();
        if input.is_empty() || input.chars().any(char::is_whitespace) {
            return None;
        }
        if input.contains('@') {
            return Some(Self {
                name: input.to_string(),
                tag: String::new(),
            });
        }
        let slash = input.rfind('/').map_or(0, |i| i + 1);
        let (name, tag) = match input[slash..].rfind(':') {
            Some(colon) => (&input[..slash + colon], &input[slash + colon + 1..]),
            None => (input, "latest"),
        };
        if name.is_empty() || tag.is_empty() {
            return None;
        }
        Some(Self {
            name: name.to_string(),
            tag: tag.to_string(),
        })
    }
}

impl fmt::Display for ImageReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.tag.is_empty() {
            write!(f, "{}", self.name)
        } else {
            write!(f, "{}:{}", self.name, self.tag)
        }
    }
}

#[cfg(test)]
mod tests;
