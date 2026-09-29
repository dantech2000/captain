//! Turns the menu model into a native `muda` menu.

use std::collections::HashMap;

use muda::accelerator::Accelerator;
use muda::{IsMenuItem, Menu, MenuId, MenuItem, PredefinedMenuItem, Submenu};

use super::menu_model::{TrayCommand, TrayItem};

/// A native menu, and the command behind each item id.
pub struct NativeMenu {
    pub menu: Menu,
    pub commands: HashMap<MenuId, TrayCommand>,
}

impl NativeMenu {
    pub fn new(items: &[TrayItem]) -> Self {
        let mut commands = HashMap::new();
        let menu = Menu::new();
        let built = build_items(items, &mut commands);
        if let Err(error) = menu.append_items(&as_refs(&built)) {
            tracing::warn!(%error, "cannot build the menu bar menu");
        }
        Self { menu, commands }
    }
}

fn build_items(
    items: &[TrayItem],
    commands: &mut HashMap<MenuId, TrayCommand>,
) -> Vec<Box<dyn IsMenuItem>> {
    items
        .iter()
        .map(|item| build_item(item, commands))
        .collect()
}

fn build_item(item: &TrayItem, commands: &mut HashMap<MenuId, TrayCommand>) -> Box<dyn IsMenuItem> {
    match item {
        TrayItem::Label(label) => Box::new(MenuItem::new(label, false, None)),
        TrayItem::Separator => Box::new(PredefinedMenuItem::separator()),
        TrayItem::Command {
            label,
            command,
            enabled,
        } => {
            // Ids are unique per menu; a rebuild replaces the whole table.
            let id = MenuId::new(format!("captain-tray-{}", commands.len()));
            let accelerator = (*command == TrayCommand::Quit)
                .then(|| "CmdOrCtrl+Q".parse::<Accelerator>().ok())
                .flatten();
            commands.insert(id.clone(), command.clone());
            Box::new(MenuItem::with_id(id, label, *enabled, accelerator))
        }
        TrayItem::Submenu { label, items } => {
            let submenu = Submenu::new(label, true);
            let children = build_items(items, commands);
            if let Err(error) = submenu.append_items(&as_refs(&children)) {
                tracing::warn!(%error, "cannot build a menu bar submenu");
            }
            Box::new(submenu)
        }
    }
}

fn as_refs(items: &[Box<dyn IsMenuItem>]) -> Vec<&dyn IsMenuItem> {
    items.iter().map(AsRef::as_ref).collect()
}
