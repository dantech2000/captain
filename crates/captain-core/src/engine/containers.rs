use super::{EngineFuture, EngineStream};
use crate::model::{
    Container, ContainerAction, ContainerDetail, EngineEvent, EngineInfo, ExecSession, ExecSpec,
    LogLine, StatsSample,
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
}
