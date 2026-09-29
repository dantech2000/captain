//! Creates, updates, and selects a context with the real `docker` CLI in a temp
//! `DOCKER_CONFIG`, then reads the store back with Captain. The user's own
//! `~/.docker` is not touched.
//! Ignored by default: `cargo test -p captain-docker --test live_contexts -- --ignored`.

use captain_core::docker_context::{context_host, meta_dir_name, read_contexts};
use captain_docker::{save_context, use_context};

const NAME: &str = "captain-agent-context";

#[test]
#[ignore = "needs the docker CLI"]
fn creates_updates_and_uses_a_context() {
    let dir = std::env::temp_dir().join(format!("captain-agent-contexts-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    save_context(&dir, NAME, "Captain test", "unix:///tmp/one.sock").unwrap();
    assert!(dir.join("contexts/meta").join(meta_dir_name(NAME)).is_dir());
    save_context(&dir, NAME, "Captain test", "ssh://me@box").unwrap();
    assert_eq!(context_host(&dir, NAME).as_deref(), Some("ssh://me@box"));

    use_context(&dir, NAME).unwrap();
    let list = read_contexts(&dir);
    assert!(list.is_current(NAME));
    assert_eq!(list.get(NAME).unwrap().description, "Captain test");

    std::fs::remove_dir_all(&dir).unwrap();
}
