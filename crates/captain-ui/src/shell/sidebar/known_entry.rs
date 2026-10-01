use captain_core::model::{ComposeProject, ProjectAction};
use captain_core::store::GroupKey;
use gpui_kit::component::Sizable;
use gpui_kit::component::menu::{ContextMenuExt, PopupMenuItem};
use gpui_kit::component::spinner::Spinner;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::help::HelpExt;
use crate::icons::{CaptainIcon, cap_icon};
use crate::new_project::open_remove_dialog;
use crate::project::up_help;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, text_button};
use crate::workspace::Workspace;

/// A project Captain knows that has no containers: its folder, Stopped, and Up.
pub fn render(
    handle: &Entity<Workspace>,
    workspace: &Workspace,
    project: ComposeProject,
    selected: bool,
    palette: &Palette,
) -> AnyElement {
    let name = project.name.clone();
    let folder = project
        .short_working_dir(std::env::home_dir().as_deref())
        .unwrap_or_default();
    let hover = palette.hover;
    let open = {
        let (handle, key) = (handle.clone(), GroupKey::Project(name.clone()));
        move |_: &ClickEvent, _: &mut Window, cx: &mut App| {
            handle.update(cx, |workspace, cx| workspace.open_group(key.clone(), cx))
        }
    };
    let action = match workspace.project_pending(&name) {
        Some(action) => div()
            .flex()
            .items_center()
            .gap(px(5.))
            .text_size(px(11.))
            .text_color(palette.text2)
            .child(Spinner::new().xsmall().color(palette.text2))
            .child(action.progress_label())
            .into_any_element(),
        None => {
            let (handle, project) = (handle.clone(), project.clone());
            text_button(
                SharedString::from(format!("sidebar-up-{name}")),
                "Up",
                ButtonTone::Accent,
                workspace.has_project_runner(),
                palette,
                move |_, _, cx| {
                    handle.update(cx, |workspace, cx| {
                        workspace.run_project_action_on(project.clone(), ProjectAction::Up, cx)
                    })
                },
            )
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .help(up_help(&name))
            .into_any_element()
        }
    };
    let entry = div()
        .id(SharedString::from(format!("sidebar-known-{name}")))
        .flex_shrink_0()
        .flex()
        .items_center()
        .gap(px(10.))
        .p(px(9.))
        .rounded(px(10.))
        .border_1()
        .cursor_pointer()
        .when(selected, |this| {
            this.bg(palette.nav_selected)
                .border_color(palette.accent.alpha(0.45))
        })
        .when(!selected, |this| {
            this.border_color(transparent_black())
                .hover(move |style| style.bg(hover))
        })
        .on_click(open)
        .child(cap_icon(
            CaptainIcon::Stack,
            px(26.),
            palette.project_color(&name).opacity(0.6),
        ))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(1.))
                .child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(palette.text2)
                        .truncate()
                        .child(name.clone()),
                )
                .child(
                    div()
                        .text_size(px(11.))
                        .text_color(palette.text3)
                        .truncate()
                        .child(format!("Stopped · {folder}")),
                ),
        )
        .child(action)
        .help(format!(
            "Show {name}. Nothing of it runs; Up starts it from {folder}. Right-click to remove it from Captain."
        ));
    with_remove_menu(entry, name, folder, handle).into_any_element()
}

/// `entry` with a right-click menu that offers Remove from Captain.
pub fn with_remove_menu(
    entry: Stateful<Div>,
    name: String,
    folder: String,
    handle: &Entity<Workspace>,
) -> impl IntoElement {
    let handle = handle.clone();
    entry.context_menu(move |menu, _, _| {
        let (name, folder, handle) = (name.clone(), folder.clone(), handle.clone());
        menu.item(PopupMenuItem::new("Remove from Captain\u{2026}").on_click(
            move |_, window, cx| {
                open_remove_dialog(name.clone(), folder.clone(), handle.clone(), window, cx)
            },
        ))
    })
}
