//! Asks the engine for its version, live, with a time limit.

use std::sync::Arc;
use std::time::Duration;

use captain_core::Engine;
use captain_core::diagnostics::EngineProbe;
use futures::future::{Either, select};
use gpui_kit::*;

use crate::workspace::Connection;

/// A socket that does not answer can hang instead of refusing.
const TIMEOUT: Duration = Duration::from_secs(5);

/// What the engine answers now. `connection` and `engine` come from the workspace.
pub async fn probe(
    connection: Connection,
    engine: Option<Arc<dyn Engine>>,
    executor: BackgroundExecutor,
) -> EngineProbe {
    let engine = match (connection, engine) {
        (Connection::Connecting, _) => return EngineProbe::Connecting,
        (Connection::Failed(error), _) => return EngineProbe::NoAnswer(error.to_string()),
        (Connection::Connected(_), None) => return EngineProbe::Connecting,
        (Connection::Connected(_), Some(engine)) => engine,
    };
    match select(engine.info(), executor.timer(TIMEOUT)).await {
        Either::Left((Ok(info), _)) => EngineProbe::Answered {
            version: info.version,
            api_version: info.api_version,
        },
        Either::Left((Err(error), _)) => EngineProbe::NoAnswer(error.to_string()),
        Either::Right(_) => {
            EngineProbe::NoAnswer(format!("no answer within {} seconds.", TIMEOUT.as_secs()))
        }
    }
}
