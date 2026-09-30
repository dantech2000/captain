/// One environment variable of a container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvVar {
    pub key: String,
    pub value: String,
}

/// Key fragments that mark a variable as a secret.
const SECRET_MARKERS: &[&str] = &[
    "SECRET", "PASSWORD", "PASSWD", "TOKEN", "API_KEY", "PRIVATE",
];

impl EnvVar {
    /// Parses a `KEY=value` entry. An entry with no `=` has an empty value.
    pub fn parse(entry: &str) -> Self {
        let (key, value) = entry.split_once('=').unwrap_or((entry, ""));
        Self {
            key: key.to_string(),
            value: value.to_string(),
        }
    }

    /// True if the key looks like it holds a secret.
    pub fn is_secret(&self) -> bool {
        is_secret_key(&self.key)
    }

    /// The value to show on screen. Secrets are masked.
    pub fn display_value(&self) -> String {
        if self.is_secret() && !self.value.is_empty() {
            "••••••••".to_string()
        } else {
            self.value.clone()
        }
    }
}

/// True if a variable or setting called `key` looks like it holds a secret.
pub fn is_secret_key(key: &str) -> bool {
    let key = key.to_ascii_uppercase();
    SECRET_MARKERS.iter().any(|marker| key.contains(marker))
}

#[cfg(test)]
mod tests;
