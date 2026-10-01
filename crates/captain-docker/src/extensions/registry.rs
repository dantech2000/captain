//! Lists a repository's tags with the OCI distribution API over HTTPS, with an
//! anonymous token when the registry asks for one. See
//! `captain_core::extension::RegistryRepository`.

use std::sync::{Arc, OnceLock};
use std::time::Duration;

use captain_core::extension::{
    HttpResponse, RegistryRepository, next_page, parse_response_parts, token_url,
};
use serde::Deserialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tokio_rustls::rustls::pki_types::ServerName;
use tokio_rustls::rustls::{ClientConfig, crypto::ring};

/// How long one request may take.
const TIMEOUT: Duration = Duration::from_secs(15);
/// At most this many pages of 1000 tags.
const MAX_PAGES: usize = 20;
/// A tag list larger than this is cut off.
const MAX_BYTES: u64 = 8 << 20;

#[derive(Deserialize)]
struct TagPage {
    #[serde(default)]
    tags: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct Token {
    #[serde(default)]
    token: String,
    #[serde(default)]
    access_token: String,
}

/// Every tag of `repository` on its registry.
pub async fn tags(repository: &str) -> Result<Vec<String>, String> {
    let repo = RegistryRepository::parse(repository)
        .ok_or_else(|| format!("\"{repository}\" is not a repository"))?;
    let mut url = repo.tags_url();
    let mut bearer = None;
    let mut tags = Vec::new();
    for _ in 0..MAX_PAGES {
        let mut response = get(&url, bearer.as_deref()).await?;
        if response.status == 401 && bearer.is_none() {
            bearer = Some(token(&response).await?);
            response = get(&url, bearer.as_deref()).await?;
        }
        if response.status != 200 {
            return Err(format!("{url} answered {}", response.status));
        }
        let page: TagPage = serde_json::from_str(&response.body)
            .map_err(|error| format!("{url} sent no tag list: {error}"))?;
        tags.extend(page.tags.unwrap_or_default());
        match response
            .header("link")
            .and_then(|link| next_page(link, &repo.host))
        {
            Some(next) => url = next,
            None => break,
        }
    }
    Ok(tags)
}

/// An anonymous token for the challenge in `response`.
async fn token(response: &HttpResponse) -> Result<String, String> {
    let url = response
        .header("www-authenticate")
        .and_then(token_url)
        .ok_or("the registry asks for a login that is not a token")?;
    let answer = get(&url, None).await?;
    if answer.status != 200 {
        return Err(format!("the registry refused a token: {}", answer.status));
    }
    let token: Token = serde_json::from_str(&answer.body)
        .map_err(|error| format!("the registry sent no token: {error}"))?;
    Ok(match token.token.is_empty() {
        true => token.access_token,
        false => token.token,
    })
}

/// One HTTPS `GET`, with `Connection: close`, read to the end. Docker Hub search
/// uses it too.
pub(crate) async fn get(url: &str, bearer: Option<&str>) -> Result<HttpResponse, String> {
    let rest = url
        .strip_prefix("https://")
        .ok_or_else(|| format!("{url} is not an https URL"))?;
    let (authority, path) = rest.split_at(rest.find('/').unwrap_or(rest.len()));
    let path = if path.is_empty() { "/" } else { path };
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) => (host, port.parse().map_err(|_| "a bad port")?),
        None => (authority, 443),
    };
    let exchange = async {
        let tcp = TcpStream::connect((host, port))
            .await
            .map_err(|error| format!("cannot reach {authority}: {error}"))?;
        let name = ServerName::try_from(host.to_string()).map_err(|error| error.to_string())?;
        let mut tls = TlsConnector::from(config()?)
            .connect(name, tcp)
            .await
            .map_err(|error| format!("TLS with {authority} failed: {error}"))?;
        let auth = bearer.map_or(String::new(), |token| {
            format!("Authorization: Bearer {token}\r\n")
        });
        let request = format!(
            "GET {path} HTTP/1.1\r\nHost: {authority}\r\nAccept: application/json\r\n\
             User-Agent: Captain\r\n{auth}Connection: close\r\n\r\n"
        );
        tls.write_all(request.as_bytes())
            .await
            .map_err(|error| error.to_string())?;
        let mut bytes = Vec::new();
        // Registries often close without a TLS close_notify; keep what came.
        let read = (&mut tls).take(MAX_BYTES).read_to_end(&mut bytes).await;
        if let Err(error) = read
            && bytes.is_empty()
        {
            return Err(error.to_string());
        }
        parse_response_parts(&bytes)
    };
    tokio::time::timeout(TIMEOUT, exchange)
        .await
        .map_err(|_| format!("{authority} did not answer in time"))?
}

/// The TLS setup with the system's trusted certificates, made once.
fn config() -> Result<Arc<ClientConfig>, String> {
    static CONFIG: OnceLock<Result<Arc<ClientConfig>, String>> = OnceLock::new();
    CONFIG
        .get_or_init(|| {
            use rustls_platform_verifier::BuilderVerifierExt;
            ClientConfig::builder_with_provider(Arc::new(ring::default_provider()))
                .with_safe_default_protocol_versions()
                .and_then(|builder| builder.with_platform_verifier())
                .map(|builder| Arc::new(builder.with_no_client_auth()))
                .map_err(|error| error.to_string())
        })
        .clone()
}
