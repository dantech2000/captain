//! Docker Desktop extensions: an image with labels and a `metadata.json` that
//! provides a web UI, a backend in the engine, and host binaries. This module has
//! the formats, the paths, the bridge messages, and the trait the Docker side
//! implements. See docs/adr/0011-extensions.md and docs/features/0025-extensions.md.

mod bridge;
mod compose;
mod id;
mod installed;
mod labels;
mod manager;
mod metadata;
mod paths;

pub use bridge::*;
pub use compose::{
    COMPOSE_FILE, GUEST_SERVICES, PROXY_IMAGE, PROXY_PORT, PROXY_SERVICE, image_project,
    with_guest_services,
};
pub use id::{PROJECT_PREFIX, data_store_id, extension_id, project_name};
pub use installed::{ExtensionCandidate, InstalledExtension, binary_name};
pub use labels::{API_VERSION_LABEL, ExtensionLabels};
pub use manager::{BridgeStream, ExtensionManager};
pub use metadata::{
    Backend, BinaryPath, DashboardTab, Exposes, ExtensionMetadata, HostSection,
    PLUGIN_IMAGE_VARIABLE, PlatformBinaries, UiSection, VmSection, host_platform,
};
pub use paths::{ExtensionPaths, MANIFEST_FILE, mime_type, ui_file};

/// The URL scheme that serves each extension's UI files: `captain-ext://<id>/`.
pub const SCHEME: &str = "captain-ext";
