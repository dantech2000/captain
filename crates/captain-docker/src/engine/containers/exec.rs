//! Interactive exec: `create_exec`, an attached `start_exec`, and `resize_exec`,
//! bridged into runtime-neutral channels. See docs/features/0011-terminal.md.

mod config;

use std::pin::Pin;
use std::time::Duration;

use bollard::Docker;
use bollard::exec::{StartExecOptions, StartExecResults};
use bollard::query_parameters::ResizeExecOptionsBuilder;
use captain_core::model::{DEFAULT_SHELLS, ExecInput, ExecResizer, ExecSession, ExecSpec};
use captain_core::{EngineError, EngineFuture};
use futures::channel::{mpsc, oneshot};
use futures::{FutureExt, StreamExt};
use tokio::io::{AsyncWrite, AsyncWriteExt};
use tokio::runtime::Handle;

use crate::{mapping, runtime};

/// How often, and how many times, to ask whether a finished exec has its exit code.
const EXIT_POLL: Duration = Duration::from_millis(25);
const EXIT_POLLS: usize = 40;

/// Starts `spec` in container `id`. Runs on the private tokio runtime.
pub fn start(
    docker: Docker,
    handle: Handle,
    id: String,
    spec: ExecSpec,
) -> EngineFuture<ExecSession> {
    let runtime = handle.clone();
    runtime::spawn(&runtime, async move {
        let command = if spec.uses_default_shell() {
            vec![default_shell(&docker, &id).await.to_string()]
        } else {
            spec.cmd.clone()
        };
        let created = docker
            .create_exec(&id, config::session_config(&spec, command.clone()))
            .await
            .map_err(mapping::engine_error)?;
        let exec_id = created.id;
        let options = StartExecOptions {
            detach: false,
            tty: spec.tty,
            output_capacity: None,
        };
        let started = docker
            .start_exec(&exec_id, Some(options))
            .await
            .map_err(mapping::engine_error)?;
        let StartExecResults::Attached { output, input } = started else {
            return Err(EngineError::Api(
                "the engine did not attach the exec".into(),
            ));
        };

        let (input_tx, input_rx) = mpsc::unbounded();
        tokio::spawn(write_input(input, input_rx));

        let (output_tx, output_rx) = mpsc::unbounded();
        let (exit_tx, exit_rx) = oneshot::channel();
        let waiter = docker.clone();
        let waited_id = exec_id.clone();
        tokio::spawn(async move {
            let mut output = output;
            while let Some(frame) = output.next().await {
                let item = frame
                    .map(|frame| frame.into_bytes().to_vec())
                    .map_err(mapping::engine_error);
                let failed = item.is_err();
                if output_tx.unbounded_send(item).is_err() || failed {
                    break;
                }
            }
            drop(output_tx);
            exit_tx.send(exit_code(&waiter, &waited_id).await).ok();
        });

        let resize_docker = docker.clone();
        let resizer = ExecResizer::new(move |cols, rows| {
            let docker = resize_docker.clone();
            let exec_id = exec_id.clone();
            runtime::spawn(&handle, async move {
                let options = ResizeExecOptionsBuilder::new()
                    .w(i32::from(cols))
                    .h(i32::from(rows))
                    .build();
                docker
                    .resize_exec(&exec_id, options)
                    .await
                    .map_err(mapping::engine_error)
            })
        });
        let exit = exit_rx
            .map(|code| code.map_err(|_| EngineError::Api("the exec was cancelled".into())))
            .boxed();

        Ok(ExecSession {
            command,
            input: ExecInput::new(input_tx),
            output: output_rx.boxed(),
            resizer,
            exit,
        })
    })
}

/// Writes queued input to the exec. When every input handle is gone, it closes the
/// write side, so the shell reads end-of-file and exits.
async fn write_input(
    mut input: Pin<Box<dyn AsyncWrite + Send>>,
    mut queued: mpsc::UnboundedReceiver<Vec<u8>>,
) {
    while let Some(bytes) = queued.next().await {
        if input.write_all(&bytes).await.is_err() || input.flush().await.is_err() {
            return;
        }
    }
    input.shutdown().await.ok();
}

/// `/bin/bash` if a quick probe exec runs it, else `/bin/sh`. The probe costs one short
/// exec, but needs no shell or `which` in the image and never shows its output.
async fn default_shell(docker: &Docker, id: &str) -> &'static str {
    let [bash, sh] = DEFAULT_SHELLS;
    if runs(docker, id, bash).await {
        bash
    } else {
        sh
    }
}

async fn runs(docker: &Docker, id: &str, shell: &str) -> bool {
    let Ok(created) = docker.create_exec(id, config::probe_config(shell)).await else {
        return false;
    };
    match docker.start_exec(&created.id, None).await {
        Ok(StartExecResults::Attached { mut output, .. }) => while output.next().await.is_some() {},
        Ok(StartExecResults::Detached) => {}
        Err(_) => return false,
    }
    exit_code(docker, &created.id).await == Some(0)
}

/// The exit code of a finished exec. The engine can report it a moment after the
/// output ends, so this asks a few times.
pub(super) async fn exit_code(docker: &Docker, exec_id: &str) -> Option<i64> {
    for _ in 0..EXIT_POLLS {
        let inspect = docker.inspect_exec(exec_id).await.ok()?;
        if inspect.running != Some(true) {
            return inspect.exit_code;
        }
        tokio::time::sleep(EXIT_POLL).await;
    }
    None
}
