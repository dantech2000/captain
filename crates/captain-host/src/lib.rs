//! Captain Engine: the machine that runs the engine, behind
//! [`captain_core::EngineHost`]. On macOS it is a Lima VM; on Linux the system
//! `dockerd`; Windows comes later. See docs/adr/0008-captain-engine.md.

mod blocking;
mod cancel;
mod host;
mod k3s;
mod lima;
pub mod machine;
pub mod probe;
mod system_host;
mod unavailable_host;

pub use host::{captain_engine_available, default_host};
pub use lima::{INSTANCE, LimaHost, LimaPaths};
pub use system_host::SystemHost;
pub use unavailable_host::UnavailableHost;
