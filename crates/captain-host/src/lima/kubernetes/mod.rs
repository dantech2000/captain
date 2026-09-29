//! Kubernetes in Captain Engine: k3s in the Lima VM with Docker as its runtime.
//! See docs/adr/0010-kubernetes.md.

mod guest;
mod install;
mod lima_kubernetes;

pub use install::{disable, install, status};
pub use lima_kubernetes::LimaKubernetes;
