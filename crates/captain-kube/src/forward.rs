//! One port forward: a listener on `127.0.0.1` that relays each connection to a pod
//! behind the Service through the Kubernetes port-forward API, as Rancher Desktop
//! does with the client library's `PortForward`. See ADR 0010.

use captain_core::HostError;
use captain_core::kubernetes::ForwardKey;
use k8s_openapi::api::core::v1::Pod;
use kube::{Api, Client};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinSet;

use crate::client::error;
use crate::target;

/// Listens on `127.0.0.1:<port>`, or a free port when it is `None`.
pub async fn bind(port: Option<u16>) -> Result<TcpListener, HostError> {
    let port = port.unwrap_or(0);
    TcpListener::bind(("127.0.0.1", port))
        .await
        .map_err(|why| HostError(format!("Cannot listen on port {port}: {why}")))
}

/// Accepts connections until the task is aborted. Dropping the task drops the
/// connection tasks too, so open connections end with the forward.
pub async fn serve(listener: TcpListener, client: Client, key: ForwardKey) {
    let mut connections = JoinSet::new();
    loop {
        while connections.try_join_next().is_some() {}
        let socket = match listener.accept().await {
            Ok((socket, _)) => socket,
            Err(error) => {
                tracing::warn!(%error, "port forward accept failed");
                continue;
            }
        };
        let (client, key) = (client.clone(), key.clone());
        connections.spawn(async move {
            if let Err(error) = relay(client, &key, socket).await {
                tracing::warn!(%error, service = %key.service, "port forward failed");
            }
        });
    }
}

/// Finds a pod for each connection, so a restarted pod is picked up. Errors while
/// the connection closes, such as a write after the pod side closed, are normal
/// for HTTP and only logged at debug level.
async fn relay(client: Client, key: &ForwardKey, mut socket: TcpStream) -> Result<(), HostError> {
    let target = target::find(&client, key).await?;
    let pods: Api<Pod> = Api::namespaced(client, &key.namespace);
    let mut forwarder = pods
        .portforward(&target.pod, &[target.port])
        .await
        .map_err(error)?;
    let mut upstream = forwarder
        .take_stream(target.port)
        .ok_or_else(|| HostError("The port-forward stream is missing.".into()))?;
    if let Err(error) = tokio::io::copy_bidirectional(&mut socket, &mut upstream).await {
        tracing::debug!(%error, "port forward connection closed");
    }
    drop(upstream);
    if let Err(error) = forwarder.join().await {
        tracing::debug!(%error, "port forward stream closed");
    }
    Ok(())
}
