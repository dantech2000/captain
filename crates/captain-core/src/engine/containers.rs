use std::path::{Path, PathBuf};

use super::{EngineFuture, EngineStream};
use crate::model::{
    Container, ContainerAction, ContainerDetail, DiskUsage, EngineEvent, EngineInfo, ExecSession,
    ExecSpec, FileEntry, FilePreview, LogLine, ProcessTable, StatsSample,
};

/// Engine info, events, and containers.
pub trait ContainerApi {
    /// The engine version, platform, and resources.
    fn info(&self) -> EngineFuture<EngineInfo>;

    /// All containers, including stopped ones.
    fn list_containers(&self) -> EngineFuture<Vec<Container>>;

    /// The engine event stream. It ends when the connection drops.
    fn events(&self) -> EngineStream<EngineEvent>;

    /// Details of one container.
    fn inspect_container(&self, id: &str) -> EngineFuture<ContainerDetail>;

    /// Live resource samples for one running container, about one a second.
    fn stats(&self, id: &str) -> EngineStream<StatsSample>;

    /// The last `tail` lines of output, then new lines as they arrive.
    fn logs(&self, id: &str, tail: usize) -> EngineStream<LogLine>;

    /// Starts, stops, restarts, or removes a container.
    fn run_action(&self, id: &str, action: ContainerAction) -> EngineFuture<()>;

    /// Runs a command in a running container, like `docker exec -it`. An empty
    /// `spec.cmd` runs the default shell: `/bin/bash` if the container has it, else
    /// `/bin/sh`. The session reports the command it picked.
    fn exec(&self, id: &str, spec: ExecSpec) -> EngineFuture<ExecSession>;

    /// The entries of the folder `path` in a running container, in no set order.
    fn list_files(&self, id: &str, path: &str) -> EngineFuture<Vec<FileEntry>>;

    /// The first `limit` bytes of the file `path`, and its full size.
    fn read_file(&self, id: &str, path: &str, limit: u64) -> EngineFuture<FilePreview>;

    /// Copies the file `path` into the host folder `dir`, or a folder as `name.tar`,
    /// like `docker cp`. It never overwrites a file; it picks a free name with
    /// [`save_name`](crate::model::save_name). Returns the new file's path.
    fn save_path(&self, id: &str, path: &str, dir: &Path) -> EngineFuture<PathBuf>;

    /// The processes of a running container, like `docker top`.
    fn top(&self, id: &str) -> EngineFuture<ProcessTable>;

    /// Sets the container's memory limit, like `docker update --memory`. The swap
    /// limit becomes twice the memory, as `docker run` sets it by default.
    fn update_memory(&self, id: &str, memory_bytes: i64) -> EngineFuture<()>;

    /// What the engine stores on its disk, by category and item, like
    /// `docker system df -v`.
    fn disk_usage(&self) -> EngineFuture<DiskUsage>;
}
