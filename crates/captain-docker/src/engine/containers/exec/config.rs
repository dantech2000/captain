use bollard::models::ExecConfig;
use captain_core::model::ExecSpec;

/// The config for the exec the user sees: `cmd` with all three streams attached.
pub fn session_config(spec: &ExecSpec, cmd: Vec<String>) -> ExecConfig {
    ExecConfig {
        attach_stdin: Some(true),
        attach_stdout: Some(true),
        attach_stderr: Some(true),
        tty: Some(spec.tty),
        // The engine takes the size as [height, width].
        console_size: spec
            .tty
            .then(|| vec![usize::from(spec.rows), usize::from(spec.cols)]),
        env: Some(spec.env_entries()),
        cmd: Some(cmd),
        ..ExecConfig::default()
    }
}

/// A quick exec that exits 0 only if `shell` exists and runs. It needs nothing else in
/// the image, not even `/bin/sh` or `which`.
pub fn probe_config(shell: &str) -> ExecConfig {
    ExecConfig {
        attach_stdout: Some(true),
        attach_stderr: Some(true),
        cmd: Some(vec![shell.to_string(), "-c".into(), "exit 0".into()]),
        ..ExecConfig::default()
    }
}

#[cfg(test)]
mod tests;
