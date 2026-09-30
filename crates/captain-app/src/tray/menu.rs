//! Turns the menu model into a native `muda` menu.

use std::collections::HashMap;

use muda::accelerator::Accelerator;
use muda::{
    CheckMenuItem, IconMenuItem, IsMenuItem, Menu, MenuId, MenuItem, PredefinedMenuItem, Submenu,
};

use super::menu_model::{TrayCommand, TrayItem};

/// A native menu, and the command behind each item id.
pub struct NativeMenu {
    pub menu: Menu,
    pub commands: HashMap<MenuId, TrayCommand>,
    /// The first status line, the engine's. Its text changes in place as the stats
    /// change, without a new menu.
    pub status: Option<IconMenuItem>,
}

impl NativeMenu {
    pub fn new(items: &[TrayItem]) -> Self {
        let mut native = Self {
            menu: Menu::new(),
            commands: HashMap::new(),
            status: None,
        };
        let built = native.build_items(items);
        if let Err(error) = native.menu.append_items(&as_refs(&built)) {
            tracing::warn!(%error, "cannot build the menu bar menu");
        }
        native
    }

    fn build_items(&mut self, items: &[TrayItem]) -> Vec<Box<dyn IsMenuItem>> {
        items.iter().map(|item| self.build_item(item)).collect()
    }

    fn build_item(&mut self, item: &TrayItem) -> Box<dyn IsMenuItem> {
        match item {
            TrayItem::Label(label) => Box::new(MenuItem::new(label, false, None)),
            TrayItem::Status { label, light } => {
                let status = IconMenuItem::new(label, false, Some(light.icon()), None);
                self.status.get_or_insert_with(|| status.clone());
                Box::new(status)
            }
            TrayItem::Separator => Box::new(PredefinedMenuItem::separator()),
            TrayItem::Command {
                label,
                command,
                enabled,
            } => {
                let id = self.add(command);
                let accelerator = (*command == TrayCommand::Quit)
                    .then(|| "CmdOrCtrl+Q".parse::<Accelerator>().ok())
                    .flatten();
                Box::new(MenuItem::with_id(id, label, *enabled, accelerator))
            }
            TrayItem::Check {
                label,
                command,
                checked,
            } => {
                let id = self.add(command);
                Box::new(CheckMenuItem::with_id(id, label, true, *checked, None))
            }
            TrayItem::Submenu {
                label,
                light,
                items,
            } => {
                let submenu = Submenu::new(label, true);
                if let Some(light) = light {
                    submenu.set_icon(Some(light.icon()));
                }
                let children = self.build_items(items);
                if let Err(error) = submenu.append_items(&as_refs(&children)) {
                    tracing::warn!(%error, "cannot build a menu bar submenu");
                }
                Box::new(submenu)
            }
        }
    }

    /// A new id for `command`. Ids are unique per menu; a rebuild replaces the table.
    fn add(&mut self, command: &TrayCommand) -> MenuId {
        let id = MenuId::new(format!("captain-tray-{}", self.commands.len()));
        self.commands.insert(id.clone(), command.clone());
        id
    }
}

fn as_refs(items: &[Box<dyn IsMenuItem>]) -> Vec<&dyn IsMenuItem> {
    items.iter().map(AsRef::as_ref).collect()
}
