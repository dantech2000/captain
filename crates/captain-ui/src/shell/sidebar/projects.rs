use captain_core::known_projects::{KnownProject, known_match, stopped_known};
use captain_core::store::{ContainerGroup, GroupKey};
use gpui_kit::assets::IconName;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::known_entry::{self, with_remove_menu};
use crate::help::{CMD, HelpExt, Hint};
use crate::icons::cap_icon;
use crate::new_project::NewProject;
use crate::project::GroupInfo;
use crate::theme::Palette;
use crate::widgets::icon_button;
use crate::workspace::{Page, Workspace};

/// "Projects" with New and an "All containers" link, then one entry per Compose
/// project, the known projects that do not run, the Kubernetes namespaces, and
/// the loose containers. It scrolls when it is long.
pub fn render(
    handle: &Entity<Workspace>,
    workspace: &Workspace,
    known: &[KnownProject],
    palette: &Palette,
) -> impl IntoElement {
    let groups = workspace.sidebar_groups();
    let live = workspace.compose_projects();
    let stopped = stopped_known(&live, known);
    let focus = (workspace.page() == Page::Project)
        .then(|| workspace.focus().cloned())
        .flatten();
    let (projects, others): (Vec<_>, Vec<_>) = groups
        .into_iter()
        .partition(|group| group.project().is_some());
    let mut entries: Vec<AnyElement> = projects
        .into_iter()
        .map(|group| {
            let selected = focus.as_ref() == Some(&group.key);
            let project = live
                .iter()
                .find(|p| Some(p.name.as_str()) == group.project());
            let known = project.filter(|p| known_match(p, known).is_some()).cloned();
            let entry = entry(handle, workspace, group, selected, palette);
            match known {
                Some(project) => {
                    let folder = project
                        .short_working_dir(std::env::home_dir().as_deref())
                        .unwrap_or_default();
                    with_remove_menu(entry, project.name, folder, handle).into_any_element()
                }
                None => entry.into_any_element(),
            }
        })
        .collect();
    entries.extend(stopped.into_iter().map(|project| {
        let selected = focus.as_ref() == Some(&GroupKey::Project(project.name.clone()));
        known_entry::render(handle, workspace, project, selected, palette)
    }));
    entries.extend(others.into_iter().map(|group| {
        let selected = focus.as_ref() == Some(&group.key);
        entry(handle, workspace, group, selected, palette).into_any_element()
    }));
    div()
        .flex_1()
        .min_h_0()
        .flex()
        .flex_col()
        .child(heading(handle, workspace, palette))
        .child(
            div()
                .id("sidebar-projects")
                .flex_1()
                .min_h_0()
                .overflow_y_scrollbar()
                .flex()
                .flex_col()
                .gap(px(4.))
                .children(entries),
        )
}

fn heading(handle: &Entity<Workspace>, workspace: &Workspace, palette: &Palette) -> Div {
    let handle = handle.clone();
    let selected = workspace.page() == Page::Containers;
    let (_, total) = workspace.shown_counts();
    let hover = palette.nav_selected;
    div()
        .flex()
        .items_center()
        .justify_between()
        .px(px(6.))
        .pb(px(6.))
        .text_size(px(11.))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.))
                .child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(palette.text3)
                        .child("Projects"),
                )
                .child(
                    icon_button(
                        "sidebar-new-project",
                        IconName::Plus,
                        Hint::with_keys(
                            "New project: run an image, start from a template, open a folder, or paste a docker run command.",
                            &[CMD, "N"],
                        ),
                        palette,
                        |_, window, cx| window.dispatch_action(Box::new(NewProject), cx),
                    )
                    .size(px(20.)),
                ),
        )
        .child(
            div()
                .id("sidebar-all-containers")
                .px(px(6.))
                .py(px(2.))
                .rounded(px(5.))
                .cursor_pointer()
                .text_color(palette.link)
                .when(selected, |this| this.bg(hover))
                .hover(move |style| style.bg(hover))
                .on_click(move |_, _, cx| {
                    handle.update(cx, |workspace, cx| {
                        workspace.clear_project_filter(cx);
                        workspace.set_page(Page::Containers, cx);
                    })
                })
                .child("All containers")
                .help(format!(
                    "Show all {total} containers in one list, grouped by project."
                )),
        )
}

fn entry(
    handle: &Entity<Workspace>,
    workspace: &Workspace,
    group: ContainerGroup,
    selected: bool,
    palette: &Palette,
) -> Stateful<Div> {
    let info = GroupInfo::of(&group, &|id| workspace.recent_crash(id).is_some(), palette);
    let handle = handle.clone();
    let key = group.key;
    let hover = palette.hover;
    div()
        .id(SharedString::from(format!("sidebar-group-{}", info.name)))
        .flex_shrink_0()
        .flex()
        .flex_col()
        .gap(px(7.))
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
        .on_click(move |_, _, cx| {
            handle.update(cx, |workspace, cx| workspace.open_group(key.clone(), cx))
        })
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(10.))
                .child(cap_icon(info.icon, px(26.), info.color))
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
                                .truncate()
                                .child(info.name.clone()),
                        )
                        .child(
                            div()
                                .text_size(px(11.))
                                .text_color(palette.text3)
                                .truncate()
                                .child(info.kind.clone()),
                        ),
                )
                .child(
                    div()
                        .flex_shrink_0()
                        .flex()
                        .items_center()
                        .gap(px(5.))
                        .text_size(px(11.))
                        .text_color(palette.text2)
                        .child(div().size(px(7.)).rounded_full().bg(info.dot))
                        .child(info.summary.clone()),
                ),
        )
        .when(!info.ports.is_empty(), |this| {
            this.child(div().flex().flex_wrap().gap(px(4.)).pl(px(36.)).children(
                info.ports.iter().take(4).map(|port| {
                    div()
                        .px(px(5.))
                        .rounded(px(5.))
                        .bg(palette.tint(palette.accent))
                        .text_color(palette.link)
                        .font_family(palette.mono())
                        .text_size(px(10.))
                        .child(format!(":{port}"))
                }),
            ))
        })
        .help(info.help)
}
