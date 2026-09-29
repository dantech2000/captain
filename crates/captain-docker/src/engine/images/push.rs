//! `docker push`: push one tag with the login from the Docker config.

use std::pin::pin;

use bollard::Docker;
use bollard::query_parameters::PushImageOptionsBuilder;
use captain_core::EngineError;
use captain_core::model::{ImageReference, PullProgress};
use captain_core::registry::{push_error, registry_host};
use futures::StreamExt;
use futures::channel::mpsc::UnboundedSender;

use crate::credentials::registry_login;
use crate::mapping;

/// Pushes `input` and sends each message to `tx`. It stops after an error, or when
/// the receiver is gone.
pub async fn push(
    docker: Docker,
    input: String,
    tx: UnboundedSender<Result<PullProgress, EngineError>>,
) {
    let Some(reference) = ImageReference::parse(&input).filter(|r| !r.tag.is_empty()) else {
        let error = EngineError::Api(format!("invalid reference format: {input:?}"));
        tx.unbounded_send(Err(error)).ok();
        return;
    };
    let host = registry_host(&reference.name);
    let login = {
        let host = host.clone();
        tokio::task::spawn_blocking(move || registry_login(&host))
            .await
            .ok()
            .flatten()
    };
    let has_login = login.is_some();
    let options = PushImageOptionsBuilder::default()
        .tag(&reference.tag)
        .build();
    let credentials = login.map(mapping::credentials);
    let mut messages = pin!(docker.push_image(&reference.name, Some(options), credentials));
    while let Some(item) = messages.next().await {
        let failed = item.is_err();
        let item =
            item.map(mapping::push_progress)
                .map_err(|error| match mapping::pull_error(error) {
                    EngineError::Api(message) => {
                        EngineError::Api(push_error(&message, &host, has_login))
                    }
                    other => other,
                });
        if tx.unbounded_send(item).is_err() || failed {
            break;
        }
    }
}
