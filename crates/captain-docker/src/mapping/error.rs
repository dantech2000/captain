use bollard::errors::Error;
use captain_core::EngineError;

/// A server response is an API error. Anything else means Captain could not talk to the engine.
pub fn engine_error(error: Error) -> EngineError {
    match error {
        Error::DockerResponseServerError { message, .. } => EngineError::Api(message),
        other => EngineError::Unreachable(other.to_string()),
    }
}

/// `None` when the engine answers 404 because the object does not exist. Any other
/// error stays an error.
pub fn found<T>(result: Result<T, Error>) -> Result<Option<T>, EngineError> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(Error::DockerResponseServerError {
            status_code: 404, ..
        }) => Ok(None),
        Err(error) => Err(engine_error(error)),
    }
}

#[cfg(test)]
mod tests;
