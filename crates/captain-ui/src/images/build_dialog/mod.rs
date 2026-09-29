//! The Build dialog: a context folder, a Dockerfile, a tag, build arguments, and a
//! target. It runs `docker buildx build` and streams the output into a log view.

mod form_state;
mod form_view;
mod log_view;
mod open;
mod pick;

pub use form_state::BuildDialog;
pub use open::open;
