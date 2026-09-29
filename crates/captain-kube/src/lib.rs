//! The Kubernetes API side of Captain: Services and port forwards, built on kube-rs.
//! kube types stay inside this crate, as bollard types stay in `captain-docker`.
//! See docs/adr/0010-kubernetes.md.

mod client;
mod clients;
mod forward;
mod forwarder;
mod services;
mod target;

pub use forwarder::KubeForwarder;
