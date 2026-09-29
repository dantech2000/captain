//! Captain Engine on Lima: one instance named `captain` in Captain's own `LIMA_HOME`.

mod args;
mod instance;
mod lima_host;
mod limactl;
mod locate;
mod paths;
mod template;
mod version;

pub use lima_host::LimaHost;
pub use paths::LimaPaths;
