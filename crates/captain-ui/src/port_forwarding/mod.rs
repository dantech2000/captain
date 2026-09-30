//! The Port Forwarding page: Kubernetes Services, grouped by namespace, and a
//! forward from a port on this Mac to each Service port. The forwards run in
//! `captain-kube` and last while Captain runs. See docs/features/0024-kubernetes.md.

mod forward_dialog;
mod forwarding_model;
mod forwarding_view;
mod service_card;

pub use forwarding_model::{ForwardingModel, forwarding_model, init};
pub use forwarding_view::PortForwardingView;
