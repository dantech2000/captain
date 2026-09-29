use captain_core::{EngineHost, HostResources};
use futures::executor::block_on_stream;

use super::LimaHost;
use crate::lima::paths::LimaPaths;

#[test]
fn a_start_refuses_while_an_unfinished_restore_left_its_backup() {
    let dir = std::env::temp_dir().join(format!("captain-restore-guard-{}", std::process::id()));
    let paths = LimaPaths {
        lima_home: dir.join("lima"),
        instance: "captain".into(),
    };
    std::fs::create_dir_all(paths.instance_dir().join(".restore-backup")).unwrap();
    let resources = HostResources {
        cpus: 2,
        memory_bytes: 1 << 30,
        disk_bytes: 1 << 30,
    };
    let host = LimaHost::with_paths(paths, resources);
    let lines: Vec<_> = block_on_stream(host.start()).collect();
    let error = lines
        .into_iter()
        .find_map(Result::err)
        .expect("the start fails");
    assert!(error.0.contains("did not finish"), "{}", error.0);
    std::fs::remove_dir_all(&dir).ok();
}
