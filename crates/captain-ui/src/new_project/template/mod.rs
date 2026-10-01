//! Start from a template: pick one of the built-in templates, set its name, ports,
//! and password, and Captain writes the project.

mod picker;
mod step;
mod view;

pub use step::TemplateStep;
