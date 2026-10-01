//! Docker Hub search and tags for the image picker. Captain sends only the search
//! text: no account, no token. See the Docker Hub section of
//! docs/features/0040-new-projects.md.

use std::time::Duration;

use futures::future::BoxFuture;
use serde::Deserialize;

/// The undocumented search endpoint the Hub's older pages use.
const SEARCH: &str = "https://hub.docker.com/v2/search/repositories/";
/// The documented tags endpoint
/// (<https://docs.docker.com/reference/api/hub/latest/>).
const TAGS: &str = "https://hub.docker.com/v2/namespaces";
/// How long to wait after a 429 without a `Retry-After`.
const DEFAULT_WAIT: Duration = Duration::from_secs(60);

/// One repository in the search results.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HubRepo {
    /// `postgres` for an official image, else `namespace/name`.
    pub name: String,
    pub description: String,
    pub stars: u64,
    pub pulls: u64,
    pub official: bool,
}

/// Why Docker Hub gave no answer.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum HubError {
    /// Too many requests; ask again after the wait.
    #[error("Docker Hub asks Captain to wait {} seconds", .0.as_secs())]
    RateLimited(Duration),
    /// Offline, a timeout, or an answer Captain cannot read.
    #[error("Docker Hub search is not available: {0}")]
    Unavailable(String),
}

/// Searches Docker Hub and lists tags. The app gives the HTTPS implementation.
pub trait DockerHub: Send + Sync + 'static {
    /// Repositories that match `query`, official images first.
    fn search(&self, query: &str) -> BoxFuture<'static, Result<Vec<HubRepo>, HubError>>;
    /// The newest tags of `repository`.
    fn tags(&self, repository: &str) -> BoxFuture<'static, Result<Vec<String>, HubError>>;
}

/// The search URL for `query`, with `size` results.
pub fn search_url(query: &str, size: usize) -> String {
    format!(
        "{SEARCH}?query={}&page_size={size}",
        percent_encode(query.trim())
    )
}

/// The tags URL for `repository`, newest first. Official images live in the
/// `library` namespace.
pub fn tags_url(repository: &str, size: usize) -> String {
    let (namespace, name) = repository
        .split_once('/')
        .unwrap_or(("library", repository));
    format!(
        "{TAGS}/{}/repositories/{}/tags?page_size={size}&ordering=last_updated",
        percent_encode(namespace),
        percent_encode(name)
    )
}

#[derive(Deserialize)]
struct SearchPage {
    #[serde(default)]
    results: Vec<SearchResult>,
}

#[derive(Deserialize)]
struct SearchResult {
    repo_name: String,
    #[serde(default)]
    short_description: Option<String>,
    #[serde(default)]
    star_count: u64,
    #[serde(default)]
    pull_count: u64,
    #[serde(default)]
    is_official: bool,
}

/// The repositories in a search answer, official images first, each group in
/// Docker Hub's order.
pub fn parse_search(body: &str) -> Result<Vec<HubRepo>, HubError> {
    let page: SearchPage = serde_json::from_str(body).map_err(unreadable)?;
    let mut repos: Vec<HubRepo> = page
        .results
        .into_iter()
        .map(|result| HubRepo {
            name: result
                .repo_name
                .strip_prefix("library/")
                .unwrap_or(&result.repo_name)
                .to_string(),
            description: result.short_description.unwrap_or_default(),
            stars: result.star_count,
            pulls: result.pull_count,
            official: result.is_official,
        })
        .collect();
    repos.sort_by_key(|repo| !repo.official);
    Ok(repos)
}

#[derive(Deserialize)]
struct TagPage {
    #[serde(default)]
    results: Vec<TagResult>,
}

#[derive(Deserialize)]
struct TagResult {
    name: String,
}

/// The tag names in a tags answer, in its order.
pub fn parse_tags(body: &str) -> Result<Vec<String>, HubError> {
    let page: TagPage = serde_json::from_str(body).map_err(unreadable)?;
    Ok(page.results.into_iter().map(|tag| tag.name).collect())
}

fn unreadable(error: serde_json::Error) -> HubError {
    HubError::Unavailable(format!("an answer Captain cannot read: {error}"))
}

/// The error for an answer with `status`: a 429 waits for `retry_after` seconds.
pub fn status_error(status: u16, retry_after: Option<&str>) -> HubError {
    if status == 429 {
        let wait = retry_after
            .and_then(|value| value.trim().parse::<u64>().ok())
            .map_or(DEFAULT_WAIT, Duration::from_secs);
        return HubError::RateLimited(wait);
    }
    HubError::Unavailable(format!("Docker Hub answered {status}"))
}

/// A count as the picker shows it: `950`, `12K`, `3.4M`, `1B`.
pub fn count_label(count: u64) -> String {
    let (value, unit) = match count {
        0..1_000 => return count.to_string(),
        1_000..1_000_000 => (count as f64 / 1e3, "K"),
        1_000_000..1_000_000_000 => (count as f64 / 1e6, "M"),
        _ => (count as f64 / 1e9, "B"),
    };
    let tenths = (value * 10.0).floor() / 10.0;
    if value >= 10.0 || tenths.fract() == 0.0 {
        format!("{}{unit}", value.floor())
    } else {
        format!("{tenths:.1}{unit}")
    }
}

/// `text` with everything but unreserved URL characters percent-encoded.
fn percent_encode(text: &str) -> String {
    text.bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (byte as char).to_string()
            }
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

#[cfg(test)]
mod tests;
