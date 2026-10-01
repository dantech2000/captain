//! Docker Hub search and tags over HTTPS, for the New sheet's image picker. See
//! `captain_core::new_project::DockerHub`.

use std::sync::OnceLock;

use captain_core::new_project::{
    DockerHub, HubError, HubRepo, parse_search, parse_tags, search_url, status_error, tags_url,
};
use futures::FutureExt;
use futures::future::BoxFuture;
use tokio::runtime::Runtime;

use crate::extensions::registry::get;
use crate::runtime;

/// Results per search.
const SEARCH_SIZE: usize = 25;
/// Tags per repository, newest first.
const TAG_SIZE: usize = 50;

/// Talks to hub.docker.com without an account.
#[derive(Debug, Clone, Copy, Default)]
pub struct DockerHubClient;

impl DockerHub for DockerHubClient {
    fn search(&self, query: &str) -> BoxFuture<'static, Result<Vec<HubRepo>, HubError>> {
        let url = search_url(query, SEARCH_SIZE);
        spawn(async move { parse_search(&fetch(&url).await?) })
    }

    fn tags(&self, repository: &str) -> BoxFuture<'static, Result<Vec<String>, HubError>> {
        let url = tags_url(repository, TAG_SIZE);
        spawn(async move { parse_tags(&fetch(&url).await?) })
    }
}

/// The body of a 200 answer from `url`.
async fn fetch(url: &str) -> Result<String, HubError> {
    let response = get(url, None).await.map_err(HubError::Unavailable)?;
    if response.status != 200 {
        return Err(status_error(
            response.status,
            response.header("retry-after"),
        ));
    }
    Ok(response.body)
}

/// Runs `future` on a small runtime of its own, made on first use.
fn spawn<T: Send + 'static>(
    future: impl Future<Output = Result<T, HubError>> + Send + 'static,
) -> BoxFuture<'static, Result<T, HubError>> {
    static RUNTIME: OnceLock<Option<Runtime>> = OnceLock::new();
    let Some(runtime) = RUNTIME.get_or_init(|| runtime::build().ok()) else {
        let error = HubError::Unavailable("Captain cannot start its network thread".into());
        return futures::future::ready(Err(error)).boxed();
    };
    runtime
        .spawn(future)
        .map(|joined| joined.unwrap_or_else(|error| Err(HubError::Unavailable(error.to_string()))))
        .boxed()
}
