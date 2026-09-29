//! The engine side of the `ddClient` bridge: backend HTTP, the three `exec` scopes,
//! and the list calls.

use std::process::Command;
use std::time::Duration;

use bollard::query_parameters::{ListContainersOptionsBuilder, ListImagesOptionsBuilder};
use captain_core::EngineError;
use captain_core::extension::{
    BridgeEvent, BridgeRequest, BridgeStream, ExecRequest, ExecScope, InstalledExtension,
    ListOptions, ServiceRequest, host_binary, parse_response, request_bytes, service_result,
};
use futures::{FutureExt, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::runtime::Handle;

use super::manager::Context;
use super::{backend, exec_policy, process};
use crate::mapping;

/// How long a backend request may take.
const SERVICE_TIMEOUT: Duration = Duration::from_secs(120);

pub fn call(
    handle: &Handle,
    context: Context,
    extension: InstalledExtension,
    request: BridgeRequest,
) -> BridgeStream {
    match request {
        BridgeRequest::Exec { scope, exec } => {
            let built = handle.spawn(async move {
                let command = command(&context, &extension, scope, &exec).await;
                (command, exec)
            });
            built
                .map(|joined| match joined {
                    Ok((Ok(command), exec)) if exec.stream => process::stream(command, exec.cmd),
                    Ok((Ok(command), exec)) => process::run(command, exec.cmd),
                    Ok((Err(error), _)) => failed(error.to_string()),
                    Err(error) => failed(error.to_string()),
                })
                .flatten_stream()
                .boxed()
        }
        other => {
            let answer = handle.spawn(async move { answer(&context, &extension, other).await });
            answer
                .map(|joined| match joined {
                    Ok(Ok(event)) => event,
                    Ok(Err(error)) => BridgeEvent::error(error.to_string()),
                    Err(error) => BridgeEvent::error(error.to_string()),
                })
                .into_stream()
                .boxed()
        }
    }
}

fn failed(message: String) -> BridgeStream {
    futures::stream::once(async move { BridgeEvent::error(message) }).boxed()
}

async fn answer(
    context: &Context,
    extension: &InstalledExtension,
    request: BridgeRequest,
) -> Result<BridgeEvent, EngineError> {
    match request {
        BridgeRequest::Service(service) => {
            let port = backend::proxy_port(&context.docker, &extension.id).await?;
            let (status, body) = tokio::time::timeout(SERVICE_TIMEOUT, fetch(port, &service))
                .await
                .map_err(|_| EngineError::Api("the extension's backend did not answer".into()))??;
            Ok(service_result(status, &body))
        }
        BridgeRequest::ListContainers(options) => list_containers(context, &options).await,
        BridgeRequest::ListImages(options) => list_images(context, &options).await,
        other => Err(EngineError::Api(format!("{other:?} is not an engine call"))),
    }
}

async fn fetch(port: u16, service: &ServiceRequest) -> Result<(u16, String), EngineError> {
    let io = |error: std::io::Error| EngineError::Api(format!("the extension's backend: {error}"));
    let mut stream = TcpStream::connect(("127.0.0.1", port)).await.map_err(io)?;
    stream
        .write_all(&request_bytes(port, service))
        .await
        .map_err(io)?;
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).await.map_err(io)?;
    parse_response(&bytes).map_err(EngineError::Api)
}

async fn list_containers(
    context: &Context,
    options: &ListOptions,
) -> Result<BridgeEvent, EngineError> {
    let mut builder = ListContainersOptionsBuilder::default()
        .all(options.all)
        .size(options.size)
        .filters(&options.filters.clone().into_iter().collect());
    if let Some(limit) = options.limit {
        builder = builder.limit(limit);
    }
    let containers = context
        .docker
        .list_containers(Some(builder.build()))
        .await
        .map_err(mapping::engine_error)?;
    to_event(&containers)
}

async fn list_images(context: &Context, options: &ListOptions) -> Result<BridgeEvent, EngineError> {
    let builder = ListImagesOptionsBuilder::default()
        .all(options.all)
        .digests(options.digests)
        .filters(&options.filters.clone().into_iter().collect());
    let images = context
        .docker
        .list_images(Some(builder.build()))
        .await
        .map_err(mapping::engine_error)?;
    to_event(&images)
}

/// The Engine API's own JSON, which is what Docker Desktop returns.
fn to_event(value: &impl serde::Serialize) -> Result<BridgeEvent, EngineError> {
    serde_json::to_value(value)
        .map(BridgeEvent::Resolve)
        .map_err(|error| EngineError::Api(error.to_string()))
}

/// The command for an exec in `scope`, always pointed at Captain's engine.
async fn command(
    context: &Context,
    extension: &InstalledExtension,
    scope: ExecScope,
    exec: &ExecRequest,
) -> Result<Command, EngineError> {
    exec_policy::check(scope, exec)?;
    let mut command = match scope {
        ExecScope::Docker => {
            let mut command = context.docker_command()?;
            command.arg(&exec.cmd).args(&exec.args);
            command
        }
        ExecScope::Vm => {
            let container = backend::backend_container(&context.docker, &extension.id).await?;
            let mut command = context.docker_command()?;
            command
                .args(["exec", &container, &exec.cmd])
                .args(&exec.args);
            command
        }
        ExecScope::Host => {
            let bin = context.paths.bin_dir(&extension.id);
            let binary =
                host_binary(&bin, &extension.binaries, &exec.cmd).map_err(EngineError::Api)?;
            let mut command = Command::new(binary);
            command.args(&exec.args).current_dir(&bin);
            command
        }
    };
    if let Some(cwd) = &exec.cwd {
        command.current_dir(cwd);
    }
    exec_policy::point_at(&mut command, &context.host, &exec.env);
    Ok(command)
}
