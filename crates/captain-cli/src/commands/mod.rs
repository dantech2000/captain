//! One file per command.

mod completion;
mod docker_env;
mod info;
mod kubernetes;
mod list_settings;
mod restart;
mod run;
mod set;
mod shell;
mod snapshot;
mod start;
mod status;
mod stop;
mod version;

pub use run::run;
