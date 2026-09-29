//! The right-hand panel for the selected image: header, actions, details, config,
//! ports, environment, labels, and layers.

mod actions;
mod header;
mod layers;
mod panel;
mod sections;

pub use panel::render;
