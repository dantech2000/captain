//! Facts about this computer for the Diagnostics page. Each probe blocks, so run
//! them on a background thread. See docs/features/0016-diagnostics.md.

mod command;
mod disk;
mod lima_version;
mod logs;
mod rosetta;

pub use command::output_within;
pub use disk::free_space;
pub use lima_version::lima_version;
pub use logs::log_bytes;
pub use rosetta::rosetta_installed;
