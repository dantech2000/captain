//! The Map tab of a project: a deterministic layout of host ports, services grouped
//! by network, and volumes; the "talks to" links read from the environment; and the
//! changes staged until Apply. See docs/features/0034-project-map.md.

mod geometry;
mod layout;
mod service;
mod staged;
mod steps;
mod talks_to;

pub use geometry::{
    Edge, EdgeKind, Lane, MapLayout, NODE_HEIGHT, NODE_WIDTH, PIN_HEIGHT, PIN_WIDTH, Placed, Rect,
    VOLUME_HEIGHT, VOLUME_WIDTH,
};
pub use layout::layout;
pub use service::MapService;
pub use staged::{Setting, StagedChange, StagedChanges, UpdatePlan};
pub use steps::{MEMORY_STEPS, memory_step, raised_memory};
pub use talks_to::{host_tokens, talks_to};
