//! Checks the text of the Docker daemon form and turns it into [`DaemonSettings`].

use serde_json::{Map, Value};

use super::DaemonSettings;
use super::merge::check_custom;

/// The lowest TCP port the form accepts; lower ports need root.
const MIN_TCP_PORT: u16 = 1024;

/// The text of each field in the form.
#[derive(Debug, Clone, Copy)]
pub struct DaemonFields<'a> {
    pub registry_mirrors: &'a str,
    pub insecure_registries: &'a str,
    pub custom: &'a str,
    pub tcp: bool,
    pub tcp_port: &'a str,
}

/// The settings for `fields`, or the first problem, for the user.
pub fn parse_fields(fields: DaemonFields<'_>) -> Result<DaemonSettings, String> {
    Ok(DaemonSettings {
        registry_mirrors: parse_mirrors(fields.registry_mirrors)?,
        insecure_registries: parse_insecure(fields.insecure_registries)?,
        custom: parse_custom(fields.custom)?,
        tcp: fields.tcp,
        tcp_port: parse_port(fields.tcp_port)?,
    })
}

/// Entries split on new lines, commas, and spaces, without duplicates.
fn entries(text: &str) -> Vec<String> {
    let mut list: Vec<String> = Vec::new();
    for entry in text.split(|c: char| c == ',' || c.is_whitespace()) {
        if !entry.is_empty() && !list.iter().any(|e| e == entry) {
            list.push(entry.to_string());
        }
    }
    list
}

/// `dockerd` rejects a mirror without a scheme.
fn parse_mirrors(text: &str) -> Result<Vec<String>, String> {
    let mirrors = entries(text);
    match mirrors
        .iter()
        .find(|m| !m.starts_with("https://") && !m.starts_with("http://"))
    {
        Some(bad) => Err(format!(
            "The registry mirror \"{bad}\" needs https:// or http://."
        )),
        None => Ok(mirrors),
    }
}

fn parse_insecure(text: &str) -> Result<Vec<String>, String> {
    let registries = entries(text);
    match registries.iter().find(|r| r.contains("://")) {
        Some(bad) => Err(format!(
            "Enter the insecure registry \"{bad}\" as host:port, without a scheme."
        )),
        None => Ok(registries),
    }
}

/// Empty text is an empty object.
fn parse_custom(text: &str) -> Result<Map<String, Value>, String> {
    if text.trim().is_empty() {
        return Ok(Map::new());
    }
    let value: Value = serde_json::from_str(text)
        .map_err(|error| format!("The custom daemon.json is not valid JSON: {error}."))?;
    let Value::Object(custom) = value else {
        return Err(
            "The custom daemon.json must be a JSON object, like {\"log-level\": \"warn\"}.".into(),
        );
    };
    check_custom(&custom)?;
    Ok(custom)
}

fn parse_port(text: &str) -> Result<u16, String> {
    text.trim()
        .parse::<u16>()
        .ok()
        .filter(|&port| port >= MIN_TCP_PORT)
        .ok_or_else(|| format!("The TCP port must be a number from {MIN_TCP_PORT} to 65535."))
}

#[cfg(test)]
mod tests;
