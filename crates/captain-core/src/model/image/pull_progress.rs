/// One message from an image pull, for example `Downloading` for one layer.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PullProgress {
    /// The layer ID for layer messages. Other messages may carry the tag here.
    pub layer: Option<String>,
    /// The status text, for example `Pulling fs layer` or `Download complete`.
    pub status: String,
    /// Bytes done in the current step, if the engine reports it.
    pub current: Option<u64>,
    /// Bytes in the current step, if the engine reports it.
    pub total: Option<u64>,
}

impl PullProgress {
    /// A message with only a status text.
    pub fn status(status: impl Into<String>) -> Self {
        Self {
            status: status.into(),
            ..Self::default()
        }
    }

    /// A message about one layer, with optional byte progress.
    pub fn layer(layer: &str, status: &str, bytes: Option<(u64, u64)>) -> Self {
        Self {
            layer: Some(layer.to_string()),
            status: status.to_string(),
            current: bytes.map(|(current, _)| current),
            total: bytes.map(|(_, total)| total),
        }
    }
}
