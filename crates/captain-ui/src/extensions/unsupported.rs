//! Linux and Windows: extensions install, but their pages cannot open. `gpui-wry`
//! has no Linux path yet, and Captain has no Windows engine. See ADR 0011.

use std::sync::Arc;

use captain_core::extension::{ExtensionManager, InstalledExtension};
use gpui_kit::App;

pub const CAN_OPEN: bool = false;

pub fn open_window(_: InstalledExtension, _: Arc<dyn ExtensionManager>, _: &mut App) {}

pub fn close_window(_: &str, _: &mut App) {}
