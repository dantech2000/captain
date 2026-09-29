//! The Run dialog: a form prefilled from an image that creates and starts a
//! container, like `docker run -d`.

mod form_state;
mod form_view;
mod open;

pub use form_state::RunDialog;
pub use open::open;
