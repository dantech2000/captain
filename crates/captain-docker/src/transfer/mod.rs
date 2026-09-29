//! Copies data between two engines for the Migration Assistant, using only the
//! Docker API. The source is read-only apart from temporary helper containers and
//! an opt-in snapshot. See docs/adr/0009-migration.md.

mod client;
mod compose;
mod container;
mod disk;
mod helper;
mod image;
mod in_flight;
mod migrator;
mod network;
mod progress;
mod scan;
mod session;
mod source;
mod verify;
mod volume;

pub use migrator::DockerMigrator;
pub use session::DockerSession;
