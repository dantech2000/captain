//! Captain Engine on Lima: one instance named `captain` in Captain's own `LIMA_HOME`.

mod args;
mod daemon;
mod instance;
mod kubernetes;
mod lima_host;
mod limactl;
mod locate;
mod paths;
mod snapshot;
mod template;
mod version;

pub use lima_host::LimaHost;
pub use locate::{current_exe, locate_limactl};
pub use paths::{INSTANCE, LimaPaths};
