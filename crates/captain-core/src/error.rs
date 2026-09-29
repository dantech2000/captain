use thiserror::Error;

/// An error from an [`Engine`](crate::Engine). It is `Clone` so views can keep it in state.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EngineError {
    /// Captain could not reach the engine at all.
    #[error("cannot reach the engine: {0}")]
    Unreachable(String),
    /// The engine answered with an error.
    #[error("engine error: {0}")]
    Api(String),
}
