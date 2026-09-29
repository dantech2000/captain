use captain_core::kubernetes::ForwardKey;
use tokio::runtime::Builder;

use super::{Running, Table, claim, lock};

#[test]
fn a_second_listener_for_one_key_stops() {
    let runtime = Builder::new_current_thread().enable_all().build().unwrap();
    let key = ForwardKey {
        namespace: "default".into(),
        service: "web".into(),
        port: 80,
    };
    let table = Table::default();
    let first = runtime.spawn(std::future::pending::<()>());
    let second = runtime.spawn(std::future::pending::<()>());

    let forward = claim(
        &table,
        key.clone(),
        Running {
            local_port: 8080,
            task: first.abort_handle(),
        },
    );
    assert_eq!(forward.local_port, 8080);
    let forward = claim(
        &table,
        key.clone(),
        Running {
            local_port: 8081,
            task: second.abort_handle(),
        },
    );

    assert_eq!(forward.local_port, 8080);
    runtime.block_on(async {
        assert!(second.await.unwrap_err().is_cancelled());
    });
    assert!(!lock(&table)[&key].task.is_finished());
}
