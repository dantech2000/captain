//! Captain Engine's Docker daemon settings: registry mirrors, insecure registries,
//! custom `daemon.json` keys, and the TCP socket. See feature 0020.

mod fields;
mod merge;
mod settings;

pub use fields::{DaemonFields, parse_fields};
pub use merge::{MANAGED_KEYS, check_custom, daemon_json};
pub use settings::{DEFAULT_TCP_PORT, DaemonSettings, DaemonState};
