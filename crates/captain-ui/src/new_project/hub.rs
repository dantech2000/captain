use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use captain_core::new_project::{DockerHub, HubRepo};
use gpui_kit::*;

/// The Docker Hub client the app installs, and what it answered this session.
/// Captain asks again for a query only after a restart, and waits after a 429.
struct Hub {
    client: Arc<dyn DockerHub>,
    searches: HashMap<String, Vec<HubRepo>>,
    tags: HashMap<String, Vec<String>>,
    /// No request before this time, after Docker Hub answered 429.
    wait_until: Option<Instant>,
}

impl Global for Hub {}

/// Installs the Docker Hub client. Without it, the image picker shows only the
/// engine's images.
pub fn set_docker_hub(cx: &mut App, client: Arc<dyn DockerHub>) {
    cx.set_global(Hub {
        client,
        searches: HashMap::new(),
        tags: HashMap::new(),
        wait_until: None,
    });
}

/// The client, unless Captain must still wait after a 429. The error says how
/// long.
pub fn client(cx: &App) -> Result<Arc<dyn DockerHub>, String> {
    let hub = cx
        .try_global::<Hub>()
        .ok_or("Docker Hub search is not available in this build.")?;
    if let Some(until) = hub.wait_until
        && let Some(left) = until.checked_duration_since(Instant::now())
    {
        return Err(format!(
            "Docker Hub asked Captain to wait. Search works again in {} seconds.",
            left.as_secs().max(1)
        ));
    }
    Ok(hub.client.clone())
}

/// The results kept for `query`.
pub fn cached_search(query: &str, cx: &App) -> Option<Vec<HubRepo>> {
    cx.try_global::<Hub>()?.searches.get(query).cloned()
}

pub fn cached_tags(repository: &str, cx: &App) -> Option<Vec<String>> {
    cx.try_global::<Hub>()?.tags.get(repository).cloned()
}

pub fn keep_search(query: String, repos: Vec<HubRepo>, cx: &mut App) {
    if cx.has_global::<Hub>() {
        let hub = cx.global_mut::<Hub>();
        hub.searches.insert(query, repos);
    }
}

pub fn keep_tags(repository: String, tags: Vec<String>, cx: &mut App) {
    if cx.has_global::<Hub>() {
        let hub = cx.global_mut::<Hub>();
        hub.tags.insert(repository, tags);
    }
}

/// Stops requests until `wait` has passed.
pub fn wait(wait: std::time::Duration, cx: &mut App) {
    if cx.has_global::<Hub>() {
        let hub = cx.global_mut::<Hub>();
        hub.wait_until = Some(Instant::now() + wait);
    }
}
