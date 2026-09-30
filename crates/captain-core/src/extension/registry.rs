//! The pieces of the OCI distribution API that list a repository's tags:
//! `GET /v2/<name>/tags/list`, the anonymous token from the `WWW-Authenticate`
//! challenge, and the `Link` header of the next page. Rancher Desktop lists tags the
//! same way (`registry.ts` in
//! <https://github.com/rancher-sandbox/rancher-desktop/tree/main/pkg/rancher-desktop/backend/containerClient>).
//! See <https://distribution.github.io/distribution/spec/api/#listing-image-tags> and
//! <https://distribution.github.io/distribution/spec/auth/token/>.

/// Docker Hub's registry API host.
const DOCKER_HUB: &str = "registry-1.docker.io";

/// A repository on its registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryRepository {
    /// The registry host, with a port if it has one.
    pub host: String,
    /// The repository path on the registry, for example `library/nginx`.
    pub path: String,
}

impl RegistryRepository {
    /// Splits a repository without tag, for example `docker/disk-usage-extension` or
    /// `ghcr.io/acme/ext`. A first part with a `.` or a `:`, or `localhost`, is the
    /// registry; anything else is on Docker Hub.
    pub fn parse(repository: &str) -> Option<Self> {
        let repository = repository.trim();
        if repository.is_empty() {
            return None;
        }
        let (host, path) = match repository.split_once('/') {
            Some((first, rest)) if first.contains(['.', ':']) || first == "localhost" => {
                (first, rest.to_string())
            }
            _ => ("docker.io", repository.to_string()),
        };
        let hub = matches!(host, "docker.io" | "index.docker.io");
        let path = if hub && !path.contains('/') {
            format!("library/{path}")
        } else {
            path
        };
        Some(Self {
            host: if hub { DOCKER_HUB } else { host }.to_string(),
            path,
        })
    }

    /// The first page of the tag list.
    pub fn tags_url(&self) -> String {
        format!("https://{}/v2/{}/tags/list?n=1000", self.host, self.path)
    }
}

/// The token URL of a `WWW-Authenticate: Bearer realm="…",service="…",scope="…"`
/// challenge. `None` for another scheme.
pub fn token_url(challenge: &str) -> Option<String> {
    let (scheme, params) = challenge.trim().split_once(' ')?;
    if !scheme.eq_ignore_ascii_case("bearer") {
        return None;
    }
    let params = challenge_params(params);
    let realm = params.iter().find(|(k, _)| k == "realm")?.1.clone();
    let query: Vec<String> = params
        .iter()
        .filter(|(key, _)| key == "service" || key == "scope")
        .map(|(key, value)| format!("{key}={}", encode(value)))
        .collect();
    Some(match query.is_empty() {
        true => realm,
        false => format!("{realm}?{}", query.join("&")),
    })
}

/// `key="value"` pairs, split on commas outside quotes, since a scope can hold one.
fn challenge_params(text: &str) -> Vec<(String, String)> {
    let mut params = Vec::new();
    let mut rest = text.trim();
    while let Some((key, after)) = rest.split_once('=') {
        let key = key
            .trim()
            .trim_start_matches(',')
            .trim()
            .to_ascii_lowercase();
        let (value, next) = match after.strip_prefix('"') {
            Some(quoted) => {
                let end = quoted.find('"').unwrap_or(quoted.len());
                (&quoted[..end], quoted.get(end + 1..).unwrap_or(""))
            }
            None => {
                let end = after.find(',').unwrap_or(after.len());
                (&after[..end], &after[end..])
            }
        };
        params.push((key, value.to_string()));
        rest = next.trim_start_matches([',', ' ']);
    }
    params
}

/// Percent-encodes a query value.
fn encode(value: &str) -> String {
    value
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b':' | b'/' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// The URL of the next page from a `Link: </v2/…?last=…&n=…>; rel="next"` header,
/// on `host`.
pub fn next_page(link: &str, host: &str) -> Option<String> {
    link.split(',').find_map(|part| {
        let (target, params) = part.split_once(';')?;
        let next = params.split(';').any(|param| {
            param
                .trim()
                .replace('"', "")
                .eq_ignore_ascii_case("rel=next")
        });
        let target = target.trim().strip_prefix('<')?.strip_suffix('>')?;
        next.then(|| match target.starts_with("https://") {
            true => target.to_string(),
            false => format!("https://{host}{target}"),
        })
    })
}

#[cfg(test)]
mod tests;
