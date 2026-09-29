//! Converts push messages and registry logins.

use bollard::auth::DockerCredentials;
use bollard::models::PushImageInfo;
use captain_core::model::PullProgress;
use captain_core::registry::RegistryAuth;

/// Converts one message of the push stream. Bollard drops the layer ID.
pub fn push_progress(info: PushImageInfo) -> PullProgress {
    let detail = info.progress_detail.unwrap_or_default();
    PullProgress {
        layer: None,
        status: info.status.unwrap_or_default(),
        current: detail.current.and_then(|n| u64::try_from(n).ok()),
        total: detail.total.and_then(|n| u64::try_from(n).ok()),
    }
}

/// The `X-Registry-Auth` credentials for a login.
pub fn credentials(login: RegistryAuth) -> DockerCredentials {
    DockerCredentials {
        username: login.username,
        password: login.password,
        identitytoken: login.identity_token,
        serveraddress: Some(login.server_address),
        ..DockerCredentials::default()
    }
}
