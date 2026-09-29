use bollard::errors::Error;
use captain_core::EngineError;

/// A server response is an API error. Anything else means Captain could not talk to the engine.
pub fn engine_error(error: Error) -> EngineError {
    match error {
        Error::DockerResponseServerError { message, .. } => EngineError::Api(message),
        other => EngineError::Unreachable(other.to_string()),
    }
}
