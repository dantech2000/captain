use thiserror::Error;

/// Why a volume or network name is not valid.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum NameError {
    #[error("Enter a name")]
    Empty,
    #[error("Use at least 2 characters")]
    TooShort,
    #[error("Start with a letter or a digit")]
    BadStart,
    #[error("\"{0}\" is not allowed. Use letters, digits, _, ., or -")]
    BadChar(char),
}

/// Checks a name against Docker's rule for volume names, `[a-zA-Z0-9][a-zA-Z0-9_.-]+`.
/// Captain uses the same rule for network names, which keeps them safe in the CLI.
pub fn validate_name(name: &str) -> Result<(), NameError> {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return Err(NameError::Empty);
    };
    if !first.is_ascii_alphanumeric() {
        return Err(NameError::BadStart);
    }
    if let Some(bad) = chars.clone().find(|c| !is_name_char(*c)) {
        return Err(NameError::BadChar(bad));
    }
    if chars.next().is_none() {
        return Err(NameError::TooShort);
    }
    Ok(())
}

fn is_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-')
}

#[cfg(test)]
mod tests;
