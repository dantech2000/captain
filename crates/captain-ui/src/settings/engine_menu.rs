//! The Engine section's two menus: the engine in use, with the other engines and
//! contexts, and "…" with start, stop, reset, and the engine's files.

use captain_core::settings::EngineChoice;
use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::component::button::{Button, ButtonCustomVariant, ButtonVariants};
use gpui_kit::component::menu::{DropdownMenu, PopupMenu, PopupMenuItem};
use gpui_kit::*;

use super::{SettingsView, engine_sheet, reset_dialog};
use crate::engine_host::HostModel;
use crate::help::HelpExt;
use crate::icons::{CaptainIcon, cap_icon};
use crate::migration::OpenMigrationAssistant;
use crate::shell::engine_name;
use crate::theme::Palette;
use crate::workspace::Connection;

/// The engine in use, what it is doing, and its state color.
fn current(view: &SettingsView, cx: &App, palette: &Palette) -> (SharedString, &'static str, Hsla) {
    if let Some(host) = view.host.as_ref().map(|host| host.read(cx))
        && host.choice(cx) == EngineChoice::Captain
    {
        let status = host.status();
        return (
            "Captain Engine".into(),
            status.label(),
            palette.host_status(status),
        );
    }
    match view.workspace.read(cx).connection() {
        Connection::Connected(info) => (
            engine_name(&info.endpoint).into(),
            "Connected",
            palette.green,
        ),
        Connection::Connecting => ("Engine".into(), "Connecting", palette.orange),
        Connection::Failed(_) => ("Engine".into(), "Not connected", palette.red),
    }
}

/// The button with the engine in use. Its menu switches engines.
pub fn engine_button(
    view: &SettingsView,
    palette: &Palette,
    cx: &mut Context<SettingsView>,
) -> Stateful<Div> {
    let (name, state, color) = current(view, cx, palette);
    let in_use = match view.workspace.read(cx).connection() {
        Connection::Connected(info) => Some(info.endpoint.clone()),
        _ => None,
    };
    let captain = view.host.clone();
    let on_captain = captain
        .as_ref()
        .is_some_and(|host| host.read(cx).choice(cx) == EngineChoice::Captain);
    let detected: Vec<(String, bool)> = view
        .detected
        .iter()
        .map(|found| {
            (
                found.host.to_string(),
                !on_captain && in_use.as_deref() == Some(&*found.host),
            )
        })
        .collect();
    let contexts: Vec<(String, String, bool)> = view
        .contexts
        .contexts
        .iter()
        .filter_map(|context| {
            let host = context.host.clone()?;
            let active = !on_captain && in_use.as_deref() == Some(host.as_str());
            Some((context.name.clone(), host, active))
        })
        .collect();
    let this = cx.weak_entity();
    let button = Button::new("settings-engine-menu")
        .custom(
            ButtonCustomVariant::new(cx)
                .color(palette.button)
                .foreground(palette.text)
                .hover(palette.nav_selected)
                .active(palette.nav_selected),
        )
        .outline()
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(10.))
                .child(cap_icon(CaptainIcon::Engine, px(16.), palette.accent_fg))
                .child(div().font_weight(FontWeight::SEMIBOLD).child(name))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(5.))
                        .text_size(px(11.5))
                        .text_color(palette.readable(color))
                        .child(div().size(px(6.)).rounded_full().bg(color))
                        .child(state),
                )
                .child(
                    Icon::new(IconName::ChevronDown)
                        .size(px(12.))
                        .text_color(palette.text3),
                ),
        )
        .dropdown_menu(move |menu, _, _| {
            engine_items(
                menu,
                captain.clone(),
                on_captain,
                &detected,
                &contexts,
                &this,
            )
        });
    div()
        .id("settings-engine")
        .child(button)
        .help("Choose the engine Captain uses: Captain Engine, another engine on this computer, or a remote host.")
}

fn engine_items(
    mut menu: PopupMenu,
    captain: Option<Entity<HostModel>>,
    on_captain: bool,
    detected: &[(String, bool)],
    contexts: &[(String, String, bool)],
    this: &WeakEntity<SettingsView>,
) -> PopupMenu {
    if let Some(host) = captain {
        menu = menu.item(
            PopupMenuItem::new("Captain Engine")
                .checked(on_captain)
                .on_click(move |_, _, cx| host.update(cx, |host, cx| host.use_captain(cx))),
        );
    }
    let use_host = |host: String| {
        let this = this.clone();
        move |_: &ClickEvent, _: &mut Window, cx: &mut App| {
            this.update(cx, |view, cx| view.use_engine(Some(host.clone()), cx))
                .ok();
        }
    };
    if !detected.is_empty() {
        menu = menu.separator().label("On this computer");
        for (host, active) in detected {
            let label = format!("{} \u{00b7} {host}", engine_name(host));
            menu = menu.item(
                PopupMenuItem::new(label)
                    .checked(*active)
                    .on_click(use_host(host.clone())),
            );
        }
    }
    if !contexts.is_empty() {
        menu = menu.separator().label("Docker contexts");
        for (name, host, active) in contexts {
            menu = menu.item(
                PopupMenuItem::new(name.clone())
                    .checked(*active)
                    .on_click(use_host(host.clone())),
            );
        }
    }
    let this = this.clone();
    menu.separator()
        .item(
            PopupMenuItem::new("Add a remote host\u{2026}").on_click(move |_, window, cx| {
                if let Some(view) = this.upgrade() {
                    engine_sheet::open(view, window, cx);
                }
            }),
        )
}

/// "…": Start, Stop, Restart, the Migration Assistant, the engine's files, Reset.
pub fn more_button(
    view: &SettingsView,
    palette: &Palette,
    cx: &mut Context<SettingsView>,
) -> Stateful<Div> {
    let host = view.host.clone().filter(|host| {
        host.read(cx).choice(cx) == EngineChoice::Captain && host.read(cx).can_control()
    });
    let workspace = view.workspace.clone();
    let button = Button::new("settings-engine-more")
        .custom(
            ButtonCustomVariant::new(cx)
                .color(palette.card)
                .foreground(palette.text3)
                .hover(palette.hover)
                .active(palette.hover),
        )
        .icon(IconName::Ellipsis)
        .dropdown_menu_with_anchor(Anchor::TopRight, move |menu, _, cx| {
            more_items(menu, host.clone(), workspace.clone(), cx)
        });
    div()
        .id("settings-engine-more-help")
        .child(button)
        .help("More engine actions: start, stop, restart, bring data from another engine, show the engine's files, and reset.")
}

fn more_items(
    menu: PopupMenu,
    host: Option<Entity<HostModel>>,
    workspace: Entity<crate::workspace::Workspace>,
    cx: &App,
) -> PopupMenu {
    let migrate = PopupMenuItem::new("Bring data from another engine\u{2026}")
        .on_click(|_, window, cx| window.dispatch_action(Box::new(OpenMigrationAssistant), cx));
    let Some(host) = host else {
        return menu
            .item(
                PopupMenuItem::new("Reconnect")
                    .on_click(move |_, _, cx| super::engine_source::reconnect(&workspace, cx)),
            )
            .item(migrate);
    };
    let model = host.read(cx);
    let status = model.status().clone();
    let idle = !status.is_busy();
    let files = model.files_dir();
    let action =
        |label: &'static str, enabled: bool, run: fn(&mut HostModel, &mut Context<HostModel>)| {
            let host = host.clone();
            PopupMenuItem::new(label)
                .disabled(!enabled)
                .on_click(move |_, _, cx| host.update(cx, run))
        };
    let reset_host = host.clone();
    menu.item(action("Start", status.can_start(), |host, cx| {
        host.start(cx)
    }))
    .item(action("Stop", status.can_stop(), |host, cx| {
        host.stop(cx).detach()
    }))
    .item(action("Restart", status.is_running(), |host, cx| {
        host.restart(cx)
    }))
    .separator()
    .item(migrate)
    .item(
        PopupMenuItem::new("Show engine files")
            .disabled(files.is_none())
            .on_click(move |_, _, cx| {
                if let Some(files) = &files {
                    cx.reveal_path(files);
                }
            }),
    )
    .separator()
    .item(
        PopupMenuItem::new("Reset engine\u{2026}")
            .disabled(!idle)
            .on_click(move |_, window, cx| reset_dialog::open(reset_host.clone(), window, cx)),
    )
}
