/// Words in a registry's answer that mean it wants a login.
const AUTH_ERRORS: &[&str] = &[
    "denied",
    "unauthorized",
    "authentication required",
    "no basic auth credentials",
];

/// The message for a failed push to `host`. When the registry wants a login, it says
/// how to log in. `has_login` is false when the Docker config has no login for `host`.
pub fn push_error(message: &str, host: &str, has_login: bool) -> String {
    let lower = message.to_lowercase();
    if !AUTH_ERRORS.iter().any(|word| lower.contains(word)) {
        return message.to_string();
    }
    let message = message.trim_end_matches('.');
    let found = if has_login {
        String::new()
    } else {
        format!(" Captain found no login for {host}.")
    };
    format!("{message}.{found} Run `docker login {host}` in a terminal, then push again.")
}

#[cfg(test)]
mod tests;
