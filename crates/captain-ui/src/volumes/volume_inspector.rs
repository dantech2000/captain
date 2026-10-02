use captain_core::format::bytes_label;
use captain_core::model::{Volume, VolumeUser};
use gpui_kit::*;

use crate::icons::CaptainIcon;
use crate::theme::Palette;
use crate::widgets::{
    DetailHeader, container_link, detail_note, detail_panel, detail_section, key_values,
    map_or_note, skeleton_lines,
};
use crate::workspace::{Page, Workspace};

/// The right-hand panel for the selected volume: its details, labels, options, and
/// the containers that mount it. `users` is `None` while they load.
pub fn render(
    volume: &Volume,
    users: Option<&Result<Vec<VolumeUser>, String>>,
    handle: &Entity<Workspace>,
    workspace: &Workspace,
    palette: &Palette,
) -> Div {
    let color = volume
        .compose_project
        .as_deref()
        .map_or(palette.accent_fg, |p| palette.project_color(p));
    let header = DetailHeader {
        icon: CaptainIcon::Volume,
        color,
        title: volume.display_name().to_string().into(),
        badge: volume
            .is_anonymous()
            .then(|| ("anonymous".into(), palette.text2)),
        subtitle: format!("{} · {}", volume.driver, volume.usage_label()).into(),
    };
    let body = div()
        .flex()
        .flex_col()
        .gap(px(18.))
        .child(detail_section("Details", details(volume, palette), palette))
        .child(detail_section(
            "Used by",
            used_by(users, handle, workspace, palette),
            palette,
        ))
        .child(detail_section(
            "Labels",
            map_or_note(&volume.labels, "No labels", palette),
            palette,
        ))
        .child(detail_section(
            "Options",
            map_or_note(&volume.options, "No driver options", palette),
            palette,
        ));
    detail_panel("volume-inspector", header, body, palette)
}

fn details(volume: &Volume, palette: &Palette) -> Div {
    let dash = |value: &str| {
        if value.is_empty() {
            "—".to_string()
        } else {
            value.to_string()
        }
    };
    let rows = vec![
        ("Name".into(), volume.name.clone()),
        ("Driver".into(), dash(&volume.driver)),
        ("Scope".into(), dash(&volume.scope)),
        ("Mountpoint".into(), dash(&volume.mountpoint)),
        ("Created".into(), dash(volume.created_date())),
        (
            "Size".into(),
            volume.size_bytes.map_or_else(|| "—".into(), bytes_label),
        ),
    ];
    key_values(rows, palette)
}

fn used_by(
    users: Option<&Result<Vec<VolumeUser>, String>>,
    handle: &Entity<Workspace>,
    workspace: &Workspace,
    palette: &Palette,
) -> Div {
    let users = match users {
        None => return skeleton_lines(2),
        Some(Err(error)) => {
            return detail_note(format!("Could not load containers: {error}"), palette);
        }
        Some(Ok(users)) if users.is_empty() => return detail_note("No containers", palette),
        Some(Ok(users)) => users,
    };
    div()
        .flex()
        .flex_col()
        .gap(px(6.))
        .children(users.iter().map(|user| {
            // The workspace follows container events, so its state is the fresher one.
            let state = workspace
                .store()
                .find(&user.container_id)
                .map_or(user.state, |c| c.state);
            let handle = handle.clone();
            let id = user.container_id.clone();
            container_link(
                SharedString::from(format!("volume-user-{}", user.container_id)),
                user.name.clone(),
                user.destination_label(),
                palette.container_state(state),
                palette,
                move |_, _, cx| {
                    handle.update(cx, |workspace, cx| {
                        workspace.select(id.clone(), cx);
                        workspace.set_page(Page::Containers, cx);
                    });
                },
            )
        }))
}
