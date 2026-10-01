//! Paste a docker run command: Captain converts it as the user types, lists what
//! it could not convert, and writes the project.

mod step;
mod view;

pub use step::PasteStep;
