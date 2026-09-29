use std::collections::HashMap;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use serde::Deserialize;

use super::RegistryAuth;
use super::host::{normalize_host, server_address};

/// The login parts of the Docker CLI's `config.json`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct DockerConfig {
    #[serde(default)]
    auths: HashMap<String, AuthEntry>,
    /// The helper for every registry, for example `osxkeychain`.
    #[serde(rename = "credsStore", default)]
    creds_store: Option<String>,
    /// Helpers for single registries, by host name.
    #[serde(rename = "credHelpers", default)]
    cred_helpers: HashMap<String, String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
struct AuthEntry {
    /// Base64 of `user:password`.
    #[serde(default)]
    auth: Option<String>,
    #[serde(default)]
    identitytoken: Option<String>,
}

/// Where the login for one registry is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CredentialSource {
    /// Run `docker-credential-<name> get` with the server address.
    Helper {
        name: String,
        server_address: String,
    },
    /// The login is in the file.
    Stored(RegistryAuth),
    /// No login.
    None,
}

impl DockerConfig {
    pub fn parse(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|err| format!("cannot read the Docker config: {err}"))
    }

    /// Where the login for `host` is, in the Docker CLI's order: `credHelpers[host]`,
    /// then `credsStore`, then the `auths` entry.
    pub fn source(&self, host: &str) -> CredentialSource {
        let server_address = server_address(host);
        let helper = self
            .cred_helpers
            .iter()
            .find(|(key, _)| normalize_host(key) == host)
            .map(|(_, name)| name.clone())
            .or_else(|| self.creds_store.clone())
            .filter(|name| !name.is_empty());
        if let Some(name) = helper {
            return CredentialSource::Helper {
                name,
                server_address,
            };
        }
        self.auths
            .iter()
            .find(|(key, _)| normalize_host(key) == host)
            .and_then(|(_, entry)| entry.login(&server_address))
            .map_or(CredentialSource::None, CredentialSource::Stored)
    }
}

impl AuthEntry {
    fn login(&self, server_address: &str) -> Option<RegistryAuth> {
        if let Some(token) = self.identitytoken.clone().filter(|t| !t.is_empty()) {
            return Some(RegistryAuth {
                identity_token: Some(token),
                server_address: server_address.to_string(),
                ..RegistryAuth::default()
            });
        }
        let decoded = STANDARD.decode(self.auth.as_deref()?.trim()).ok()?;
        let decoded = String::from_utf8(decoded).ok()?;
        let (username, password) = decoded.split_once(':')?;
        Some(RegistryAuth {
            username: Some(username.to_string()),
            password: Some(password.to_string()),
            server_address: server_address.to_string(),
            ..RegistryAuth::default()
        })
    }
}

#[cfg(test)]
mod tests;
