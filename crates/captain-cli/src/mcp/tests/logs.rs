//! The `logs` service filter, strict arguments, and the `wait_for_healthy`
//! deadline.

use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use captain_core::Engine;
use serde_json::json;

use super::super::{Connect, Connected, Source};
use super::{call, client, enabled, fake, read, serve, text};

#[tokio::test]
async fn project_logs_narrow_to_one_service() {
    let client = client(fake()).await;
    let result = call(
        &client,
        "logs",
        json!({ "project": "shop", "service": "web" }),
    )
    .await;
    let output = text(&result);
    assert!(output.contains("web | "), "{output}");
    assert!(!output.contains("api | "), "{output}");
    let unknown = call(
        &client,
        "logs",
        json!({ "project": "shop", "service": "worker" }),
    )
    .await;
    assert_eq!(unknown.is_error, Some(true));
    assert!(text(&unknown).contains("Services: api, web."));
}

#[tokio::test]
async fn an_unknown_argument_is_refused_with_the_known_ones() {
    let client = client(fake()).await;
    let result = call(
        &client,
        "logs",
        json!({ "project": "shop", "servce": "web" }),
    )
    .await;
    assert_eq!(result.is_error, Some(true));
    let why = text(&result);
    assert!(
        why.contains("unknown field `servce`") && why.contains("`service`"),
        "{why}"
    );
}

#[tokio::test]
async fn wait_for_healthy_keeps_its_deadline_when_the_engine_hangs() {
    let (release, hold) = mpsc::channel::<()>();
    let hold = Mutex::new(hold);
    let engine: Arc<dyn Engine> = Arc::new(fake());
    let connect: Connect = Arc::new(move || {
        hold.lock().unwrap().recv().ok();
        Ok(Connected {
            engine: engine.clone(),
            runner: None,
        })
    });
    let client =
        serve(Source::new("Other engine", None, connect, false).with_settings(read(enabled())))
            .await;
    let result = call(
        &client,
        "wait_for_healthy",
        json!({ "container": "shop-web-1", "timeout_seconds": 0 }),
    )
    .await;
    release.send(()).ok();
    let report = result.structured_content.unwrap();
    assert_eq!(report["ready"], false);
    assert!(
        report["reason"]
            .as_str()
            .unwrap()
            .contains("did not answer")
    );
}
