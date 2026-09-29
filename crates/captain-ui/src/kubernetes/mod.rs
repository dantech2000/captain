//! Kubernetes in the UI: the model behind the Kubernetes card in Settings, the Port
//! Forwarding page, and the sidebar link. The cluster itself is in `captain-host`.
//! See docs/features/0024-kubernetes.md.

mod kube_actions;
mod kube_event;
mod kube_model;

pub use kube_event::KubeEvent;
pub use kube_model::{KubernetesModel, init, kubernetes_model};
