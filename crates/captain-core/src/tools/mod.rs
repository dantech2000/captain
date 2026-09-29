//! The command-line tools Captain runs: `limactl`, `docker`, and the docker CLI
//! plugins. A packaged Captain ships its own copies; a dev build finds them on
//! `PATH`. See docs/features/0021-packaging.md.

mod bundle;
mod docker_config;

pub use bundle::{Bundle, locate_tool};
pub use docker_config::docker_config;
