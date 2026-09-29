use captain_core::model::ExecSpec;

use super::{probe_config, session_config};

#[test]
fn the_session_attaches_everything_and_sets_the_size() {
    let config = session_config(&ExecSpec::shell(100, 30), vec!["/bin/sh".into()]);
    assert_eq!(config.attach_stdin, Some(true));
    assert_eq!(config.attach_stdout, Some(true));
    assert_eq!(config.tty, Some(true));
    assert_eq!(config.console_size, Some(vec![30, 100]));
    assert_eq!(config.cmd, Some(vec!["/bin/sh".to_string()]));
    let env = config.env.expect("env");
    assert!(env.contains(&"TERM=xterm-256color".to_string()));
}

#[test]
fn the_probe_runs_the_shell_without_input() {
    let config = probe_config("/bin/bash");
    assert_eq!(config.attach_stdin, None);
    assert_eq!(config.tty, None);
    assert_eq!(
        config.cmd,
        Some(vec!["/bin/bash".into(), "-c".into(), "exit 0".into()])
    );
}
