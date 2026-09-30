//! The ⌘K command palette.

mod command;
mod command_palette;
mod commands;
mod footer;
mod grammar;
mod key_hint;
mod keys;
mod ranking;
mod results;
mod row_help;
mod run_action;
mod search_field;
mod suggestion_row;
mod try_row;

pub use command_palette::CommandPalette;
pub(crate) use key_hint::key_hint;
pub use keys::init;

gpui_kit::actions!(captain, [ToggleCommandPalette]);
