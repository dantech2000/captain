//! Docker Desktop extensions: an image with labels and a `metadata.json` that
//! provides a web UI, a backend in the engine, and host binaries. This module has
//! the formats, the paths, the bridge messages, and the trait the Docker side
//! implements. See docs/adr/0011-extensions.md and docs/features/0025-extensions.md.

mod bridge;
mod compose;
mod compose_refs;
mod compose_walk;
mod id;
mod installed;
mod labels;
mod manager;
mod marker;
mod metadata;
mod paths;
mod registry;
mod tags;
mod update;

pub use bridge::*;
pub use compose::{
    COMPOSE_FILE, GUEST_SERVICES, PROXY_IMAGE, PROXY_PORT, PROXY_SERVICE, image_project,
    with_guest_services,
};
pub use compose_refs::compose_references;
pub use compose_walk::ComposeWalk;
pub use id::{PROJECT_PREFIX, data_store_id, extension_id, image_repository, project_name};
pub use installed::{ExtensionCandidate, InstalledExtension, binary_name, engine_key};
pub use labels::{API_VERSION_LABEL, ExtensionLabels};
pub use manager::{BridgeStream, ExtensionManager};
pub use marker::{EXTENSION_LABEL, backend_extension, is_backend_project};
pub use metadata::{
    Backend, BinaryPath, DashboardTab, Exposes, ExtensionMetadata, HostSection,
    PLUGIN_IMAGE_VARIABLE, PlatformBinaries, UiSection, VmSection, host_platform,
};
pub use paths::{ExtensionPaths, MANIFEST_FILE, mime_type, ui_file};
pub use registry::{RegistryRepository, next_page, token_url};
pub use tags::{FALLBACK_TAG, choose_tag, newest_version_tag, tag_version, untagged_repository};
pub use update::{
    DEFAULT_UPDATE_TAG, ExtensionUpdate, UpdateCheck, compare_versions, update_reference,
};

/// The URL scheme that serves each extension's UI files: `captain-ext://<id>/`.
pub const SCHEME: &str = "captain-ext";
