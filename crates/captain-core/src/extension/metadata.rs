//! `metadata.json`, the file at the root of every extension image. See
//! <https://docs.docker.com/extensions/extensions-sdk/architecture/metadata/>.

use serde::{Deserialize, Serialize};

/// The placeholder for the extension's own image in `vm.image` and Compose files.
pub const PLUGIN_IMAGE_VARIABLE: &str = "${DESKTOP_PLUGIN_IMAGE}";

/// What an extension provides. Every part is optional.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtensionMetadata {
    /// The icon, as a path in the image.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ui: Option<UiSection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vm: Option<VmSection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<HostSection>,
}

/// The UI. Docker Desktop has one extension point, the dashboard tab.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiSection {
    #[serde(
        rename = "dashboard-tab",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub dashboard_tab: Option<DashboardTab>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardTab {
    pub title: String,
    /// The folder in the image that holds the UI files.
    pub root: String,
    /// The page to load, relative to `root`.
    pub src: String,
}

/// The backend that runs in the engine: one image, or a Compose file in the image.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VmSection {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub composefile: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exposes: Option<Exposes>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Exposes {
    /// A socket name in `/run/guest-services`.
    pub socket: String,
}

/// Binaries that Captain copies to the host.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostSection {
    #[serde(default)]
    pub binaries: Vec<PlatformBinaries>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformBinaries {
    #[serde(default)]
    pub darwin: Vec<BinaryPath>,
    #[serde(default)]
    pub linux: Vec<BinaryPath>,
    #[serde(default)]
    pub windows: Vec<BinaryPath>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BinaryPath {
    pub path: String,
}

/// How the backend runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Backend {
    /// One image. The placeholder is already replaced.
    Image(String),
    /// A Compose file, as a path in the extension image.
    Compose(String),
}

impl ExtensionMetadata {
    /// Parses `metadata.json`. Fails when the file is not JSON, or when `vm` names
    /// neither an image nor a Compose file.
    pub fn parse(json: &str) -> Result<Self, String> {
        let metadata: Self = serde_json::from_str(json)
            .map_err(|error| format!("metadata.json is not valid: {error}"))?;
        if let Some(vm) = &metadata.vm
            && vm.image.is_none()
            && vm.composefile.is_none()
        {
            return Err("metadata.json has a vm section without an image or a composefile".into());
        }
        Ok(metadata)
    }

    pub fn dashboard_tab(&self) -> Option<&DashboardTab> {
        self.ui.as_ref()?.dashboard_tab.as_ref()
    }

    /// The backend, with `${DESKTOP_PLUGIN_IMAGE}` replaced by `image`. An image wins
    /// over a Compose file, as in Docker Desktop.
    pub fn backend(&self, image: &str) -> Option<Backend> {
        let vm = self.vm.as_ref()?;
        match (&vm.image, &vm.composefile) {
            (Some(backend), _) => Some(Backend::Image(
                backend.replace(PLUGIN_IMAGE_VARIABLE, image),
            )),
            (None, Some(file)) => Some(Backend::Compose(file.clone())),
            (None, None) => None,
        }
    }

    /// The socket the backend listens on, in `/run/guest-services`.
    pub fn backend_socket(&self) -> Option<&str> {
        let socket = self.vm.as_ref()?.exposes.as_ref()?.socket.as_str();
        (!socket.is_empty()).then_some(socket)
    }

    /// The image paths of the host binaries for `platform`: `darwin`, `linux`, or
    /// `windows`.
    pub fn host_binaries(&self, platform: &str) -> Vec<String> {
        let Some(host) = &self.host else {
            return Vec::new();
        };
        host.binaries
            .iter()
            .flat_map(|binaries| match platform {
                "darwin" => binaries.darwin.as_slice(),
                "linux" => binaries.linux.as_slice(),
                "windows" => binaries.windows.as_slice(),
                _ => &[],
            })
            .map(|binary| binary.path.clone())
            .collect()
    }
}

/// The platform name that `metadata.json` uses for this computer.
pub fn host_platform() -> &'static str {
    if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(windows) {
        "windows"
    } else {
        "linux"
    }
}

#[cfg(test)]
mod tests;
