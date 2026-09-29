//! Snapshots of the stopped Captain Engine instance: clones of its files in
//! `~/.captain/snapshots/<uuid>`. See docs/adr/0012-snapshots.md.

mod copy;
mod folder;
mod lima_snapshots;
mod plan;
mod steps;
mod swap;

pub use lima_snapshots::LimaSnapshots;
