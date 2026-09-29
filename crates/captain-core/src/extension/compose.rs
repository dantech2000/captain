//! The Compose project that runs an extension's backend. Captain writes it as JSON,
//! which Compose reads like YAML. Every service gets the `/run/guest-services`
//! volume, and a proxy service publishes the backend's socket on `127.0.0.1`, as in
//! Rancher Desktop. See docs/adr/0011-extensions.md.

use serde_json::{Map, Value, json};

use super::project_name;

/// Where backends put their sockets.
pub const GUEST_SERVICES: &str = "/run/guest-services";
/// The proxy service in every backend project with a socket.
pub const PROXY_SERVICE: &str = "captain-proxy";
/// socat forwards TCP to the backend's Unix socket.
pub const PROXY_IMAGE: &str = "alpine/socat:1.8.1.3";
/// The port the proxy listens on inside its container.
pub const PROXY_PORT: u16 = 8080;
/// The file Captain writes in the extension's `compose` folder.
pub const COMPOSE_FILE: &str = "captain-compose.json";
const VOLUME: &str = "guest-services";

/// The project for a backend that is one image.
pub fn image_project(id: &str, image: &str, socket: Option<&str>) -> Value {
    let config = json!({ "services": { "backend": { "image": image } } });
    with_guest_services(config, id, socket)
}

/// Takes a project from `docker compose config --format json` and adds the name, the
/// volume, the proxy, and a restart policy, so the backend comes back when the
/// engine restarts.
pub fn with_guest_services(mut config: Value, id: &str, socket: Option<&str>) -> Value {
    let Some(project) = config.as_object_mut() else {
        return config;
    };
    project.insert("name".into(), Value::String(project_name(id)));
    let services = project
        .entry("services")
        .or_insert_with(|| Value::Object(Map::new()));
    if let Some(services) = services.as_object_mut() {
        for service in services.values_mut().filter_map(Value::as_object_mut) {
            add_volume(service);
            service
                .entry("restart")
                .or_insert_with(|| "unless-stopped".into());
        }
        if let Some(socket) = socket {
            services.insert(PROXY_SERVICE.into(), proxy(socket));
        }
    }
    let volumes = project
        .entry("volumes")
        .or_insert_with(|| Value::Object(Map::new()));
    if let Some(volumes) = volumes.as_object_mut() {
        volumes.insert(VOLUME.into(), json!({}));
    }
    config
}

/// Mounts the guest services volume, unless the service mounts something there.
fn add_volume(service: &mut Map<String, Value>) {
    let volumes = service
        .entry("volumes")
        .or_insert_with(|| Value::Array(Vec::new()));
    let Some(volumes) = volumes.as_array_mut() else {
        return;
    };
    let mounted = volumes.iter().any(|volume| match volume {
        Value::String(short) => short.split(':').nth(1) == Some(GUEST_SERVICES),
        other => other.get("target").and_then(Value::as_str) == Some(GUEST_SERVICES),
    });
    if !mounted {
        volumes.push(guest_volume());
    }
}

fn guest_volume() -> Value {
    json!({ "type": "volume", "source": VOLUME, "target": GUEST_SERVICES })
}

fn proxy(socket: &str) -> Value {
    json!({
        "image": PROXY_IMAGE,
        "command": [
            format!("TCP-LISTEN:{PROXY_PORT},fork,reuseaddr"),
            format!("UNIX-CONNECT:{GUEST_SERVICES}/{socket}"),
        ],
        "ports": [format!("127.0.0.1::{PROXY_PORT}")],
        "volumes": [guest_volume()],
        "restart": "unless-stopped",
    })
}

#[cfg(test)]
mod tests;
