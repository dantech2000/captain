//! The ⌘K command palette.

mod command;
mod command_palette;
mod commands;
mod footer;
mod key_hint;
mod keys;
mod ranking;
mod results;
mod search_field;

pub use command_palette::CommandPalette;
pub(crate) use key_hint::key_hint;
pub use keys::init;

gpui_kit::actions!(captain, [ToggleCommandPalette]);
