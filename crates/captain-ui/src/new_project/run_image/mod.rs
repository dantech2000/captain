//! Run an image: pick an image from this engine or Docker Hub, set its ports,
//! environment, volumes, and restart policy, then save it as a one-service
//! Compose project or run it as a plain container, like `docker run -d`.

mod choice;
mod controls_view;
mod form_state;
mod form_view;
mod picker;
mod picker_rows;
mod rows_view;
mod submit;

pub use form_state::RunImage;
