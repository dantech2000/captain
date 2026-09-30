//! The Project page: one sidebar entry with its services, ports, tasks, and one log
//! for all of them. See docs/features/0030-project-window.md.

mod action_help;
mod card_actions;
mod card_note;
mod files;
mod group_info;
mod header;
mod log_rows;
mod log_view;
mod map;
mod notice;
mod open_row;
mod page;
mod project_view;
mod service_card;
mod staging;
mod system_open;
mod tasks;
mod tasks_card;
mod view_tabs;

pub(crate) use action_help::{down_help, restart_help, up_help};
pub use group_info::GroupInfo;
pub use notice::ProjectNotice;
pub use project_view::ProjectView;
