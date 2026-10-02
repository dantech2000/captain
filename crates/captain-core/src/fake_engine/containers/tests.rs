use futures::executor::block_on;

use futures::StreamExt;

use crate::model::{Container, ContainerAction, ContainerState, ExecSpec};
use crate::{ContainerApi, FakeEngine};

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
            kube_namespace: None,
            extension: None,
        }],
        ..FakeEngine::default()
    }
}

#[test]
fn only_force_remove_deletes_a_running_container() {
    let run = |state, action| block_on(engine_with(state).run_action("web", action));
    assert!(run(ContainerState::Running, ContainerAction::Remove).is_err());
    assert!(run(ContainerState::Running, ContainerAction::ForceRemove).is_ok());
    assert!(run(ContainerState::Exited, ContainerAction::Remove).is_ok());
}

#[test]
fn exec_echoes_input_until_the_input_is_dropped() {
    let engine = engine_with(ContainerState::Running);
    let session = block_on(engine.exec("web", ExecSpec::shell(80, 24))).expect("exec");
    session.input.send(b"echo hi\r".to_vec()).expect("send");
    let mut output = session.output;
    let first = block_on(output.next()).expect("a chunk").expect("ok");
    assert_eq!(first, b"echo hi\r");
    drop(session.input);
    assert!(block_on(output.next()).is_none());
    assert_eq!(block_on(session.exit), Ok(Some(0)));
}
