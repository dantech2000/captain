//! Kubernetes in Captain Engine: k3s with Docker as its runtime. This module has the
//! settings, the version list, the download names, the kubeconfig merge, and the
//! traits the host and the port forwarder implement. See docs/adr/0010-kubernetes.md
//! and docs/features/0024-kubernetes.md.

mod assets;
mod host;
mod kubeconfig;
mod kubeconfig_files;
mod services;
mod settings;
mod version;
mod versions;

pub use assets::{K3sAssets, download_url, expected_sha256};
pub use host::{KubernetesHost, KubernetesStatus};
pub use kubeconfig::{
    CONTEXT, captain_config, contexts, current_context, merge, parse, remove_captain, set_current,
    to_yaml,
};
pub use kubeconfig_files::{
    KubeContexts, install_captain, kubeconfig_paths, load_contexts, read_config, uninstall_captain,
    use_context, user_kubeconfig_paths, write_config,
};
pub use services::{
    Forward, ForwardKey, KubeService, PortForwarding, ServicePort, TargetPort, check_local_port,
    resolve_target,
};
pub use settings::{DEFAULT_PORT, KubernetesSettings, RECOMMENDED_MEMORY};
pub use version::{K3sVersion, MINIMUM_VERSION};
pub use versions::{VersionList, parse_channels, parse_releases};
