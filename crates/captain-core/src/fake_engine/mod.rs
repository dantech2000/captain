//! An in-memory [`Engine`](crate::Engine) for tests and UI previews. Each resource
//! keeps its data and trait impl in its own file.

mod containers;
mod images;
mod networks;
mod volumes;

pub use containers::FakeFiles;
pub use images::FakeImages;
pub use networks::FakeNetworks;
pub use volumes::FakeVolumes;

use crate::model::{Container, EngineEvent, EngineInfo, LogLine, ProcessTable, StatsSample};

#[derive(Debug, Clone, Default)]
pub struct FakeEngine {
    pub info: Option<EngineInfo>,
    pub containers: Vec<Container>,
    pub events: Vec<EngineEvent>,
    pub stats: Vec<StatsSample>,
    pub logs: Vec<LogLine>,
    pub files: FakeFiles,
    /// What `top` returns for a running container.
    pub processes: ProcessTable,
    pub images: FakeImages,
    pub volumes: FakeVolumes,
    pub networks: FakeNetworks,
}
