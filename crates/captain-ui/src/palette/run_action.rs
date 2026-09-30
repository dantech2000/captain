use captain_core::grammar::{Action, Destination};
use captain_core::model::PortLink;
use captain_core::store::GroupKey;
use gpui_kit::component::WindowExt;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;

use super::command::run_project;
use crate::menu_bar::open_float_log;
use crate::port_forwarding::forwarding_model;
use crate::workspace::{InspectorTab, LogFilter, Page, Workspace};

/// Runs a command of the grammar. Down still asks first.
pub fn run(action: Action, workspace: &Entity<Workspace>, cx: &mut App) {
    match action {
        Action::Containers { ids, action } => {
            workspace.update(cx, |w, cx| w.run_actions(ids, action, cx))
        }
        Action::Project { name, action } => run_project(name, action, workspace, cx),
        Action::Logs { id, since, errors } => workspace.update(cx, |w, cx| {
            show_entry_of(w, &id, cx);
            w.open_logs(id, LogFilter { since, errors }, cx);
        }),
        Action::ProjectLog(project) => {
            workspace.update(cx, |w, cx| w.open_group(GroupKey::Project(project), cx))
        }
        Action::Shell(id) => workspace.update(cx, |w, cx| {
            show_entry_of(w, &id, cx);
            w.open_card_tab(id, InspectorTab::Terminal, cx);
        }),
        Action::Float(id) => {
            let name = workspace
                .read(cx)
                .store()
                .find(&id)
                .map(|c| c.name.clone())
                .unwrap_or_default();
            open_float_log(workspace.clone(), id, name, cx);
        }
        Action::Open(PortLink::Open(url)) => cx.open_url(&url),
        Action::Open(PortLink::Copy { address, .. }) => copy(address, cx),
        Action::Forward { key, local_port } => {
            forwarding_model(cx).update(cx, |model, cx| model.forward(key, local_port, cx));
            workspace.update(cx, |w, cx| w.set_page(Page::PortForwarding, cx));
        }
        Action::Go(page) => workspace.update(cx, |w, cx| w.set_page(page_of(page), cx)),
    }
}

/// The page for a page word of the grammar.
pub fn page_of(page: Destination) -> Page {
    match page {
        Destination::Containers => Page::Containers,
        Destination::Images => Page::Images,
        Destination::Volumes => Page::Volumes,
        Destination::Networks => Page::Networks,
        Destination::Extensions => Page::Extensions,
        Destination::Snapshots => Page::Snapshots,
        Destination::Storage => Page::Storage,
        Destination::PortForwarding => Page::PortForwarding,
        Destination::Diagnostics => Page::Diagnostics,
        Destination::Settings => Page::Settings,
    }
}

/// Shows the Project page of the sidebar entry that holds container `id`.
fn show_entry_of(workspace: &mut Workspace, id: &str, cx: &mut Context<Workspace>) {
    if let Some(key) = workspace.store().find(id).map(GroupKey::of) {
        workspace.open_group(key, cx);
    }
}

/// Copies the address of a port that does not serve web pages, as the Open row does.
fn copy(address: String, cx: &mut App) {
    cx.write_to_clipboard(ClipboardItem::new_string(address.clone()));
    let Some(window) = cx.active_window() else {
        return;
    };
    // The palette runs commands while it handles a key or click in this window.
    cx.defer(move |cx| {
        window
            .update(cx, |_, window, cx| {
                window.push_notification(Notification::success(format!("Copied {address}.")), cx);
            })
            .ok();
    });
}
