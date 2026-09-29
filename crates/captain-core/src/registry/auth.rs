use serde::Deserialize;

/// The user name a credential helper returns for an identity token.
const TOKEN_USERNAME: &str = "<token>";

/// A registry login, as the engine takes it in the `X-Registry-Auth` header.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RegistryAuth {
    pub username: Option<String>,
    pub password: Option<String>,
    /// An OAuth identity token, used instead of a user name and password.
    pub identity_token: Option<String>,
    pub server_address: String,
}

/// What `docker-credential-<name> get` prints.
#[derive(Deserialize)]
struct HelperOutput {
    #[serde(rename = "Username", default)]
    username: String,
    #[serde(rename = "Secret", default)]
    secret: String,
}

impl RegistryAuth {
    /// Reads a credential helper's output for `server_address`. The user name
    /// `<token>` means the secret is an identity token. `None` for output that is not
    /// a login.
    pub fn from_helper(stdout: &str, server_address: &str) -> Option<Self> {
        let output: HelperOutput = serde_json::from_str(stdout.trim()).ok()?;
        if output.secret.is_empty() {
            return None;
        }
        let server_address = server_address.to_string();
        Some(if output.username == TOKEN_USERNAME {
            Self {
                identity_token: Some(output.secret),
                server_address,
                ..Self::default()
            }
        } else {
            Self {
                username: Some(output.username),
                password: Some(output.secret),
                server_address,
                ..Self::default()
            }
        })
    }
}

#[cfg(test)]
mod tests;
