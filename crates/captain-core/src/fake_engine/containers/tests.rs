use futures::executor::block_on;

use futures::StreamExt;

use crate::model::{Container, ContainerAction, ContainerState, ExecSpec};
use crate::{ContainerApi, EngineError, FakeEngine};

fn engine_with(state: ContainerState) -> FakeEngine {
    FakeEngine {
        containers: vec![Container {
            id: "web".into(),
            name: "web".into(),
            image: "nginx:latest".into(),
            state,
            status: String::new(),
            ports: Vec::new(),
            created: 0,
            compose_project: None,
            compose: Default::default(),
            health: None,
        }],
        ..FakeEngine::default()
    }
}

fn run(state: ContainerState, action: ContainerAction) -> Result<(), EngineError> {
    block_on(engine_with(state).run_action("web", action))
}

#[test]
fn pause_and_unpause_follow_the_state() {
    assert!(run(ContainerState::Running, ContainerAction::Pause).is_ok());
    assert!(run(ContainerState::Exited, ContainerAction::Pause).is_err());
    assert!(run(ContainerState::Paused, ContainerAction::Unpause).is_ok());
    assert!(run(ContainerState::Running, ContainerAction::Unpause).is_err());
}

#[test]
fn only_force_remove_deletes_a_running_container() {
    assert!(run(ContainerState::Running, ContainerAction::Remove).is_err());
    assert!(run(ContainerState::Running, ContainerAction::ForceRemove).is_ok());
    assert!(run(ContainerState::Exited, ContainerAction::Remove).is_ok());
}

#[test]
fn an_unknown_container_fails() {
    let result = block_on(FakeEngine::default().run_action("nope", ContainerAction::Stop));
    assert!(matches!(result, Err(EngineError::Api(_))));
}

#[test]
fn exec_echoes_input_until_the_input_is_dropped() {
    let engine = engine_with(ContainerState::Running);
    let session = block_on(engine.exec("web", ExecSpec::shell(80, 24))).expect("exec");
    assert_eq!(session.command_line(), "/bin/bash");
    block_on(session.resizer.resize(100, 30)).expect("resize");
    session.input.send(b"echo hi\r".to_vec()).expect("send");
    let mut output = session.output;
    let first = block_on(output.next()).expect("a chunk").expect("ok");
    assert_eq!(first, b"echo hi\r");
    drop(session.input);
    assert!(block_on(output.next()).is_none());
    assert_eq!(block_on(session.exit), Ok(Some(0)));
}

#[test]
fn exec_needs_a_running_container() {
    let engine = engine_with(ContainerState::Exited);
    assert!(block_on(engine.exec("web", ExecSpec::shell(80, 24))).is_err());
    assert!(block_on(engine.exec("nope", ExecSpec::shell(80, 24))).is_err());
}
