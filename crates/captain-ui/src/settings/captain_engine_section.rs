//! The Captain Engine card: which engine to use, and Captain Engine's state,
//! resources, quit behavior, migration, and reset. See ADR 0008.

use captain_core::HostStatus;
use captain_core::settings::{EngineChoice, Settings};
use gpui_kit::component::switch::Switch;
use gpui_kit::*;

use super::{engine_resources, reset_dialog, store};
use crate::engine_host::HostModel;
use crate::help::HelpExt;
use crate::migration::OpenMigrationAssistant;
use crate::theme::Palette;
use crate::widgets::{
    ButtonTone, Segment, pill, segmented, settings_card, settings_row, text_button,
};

pub fn render(model: &Entity<HostModel>, settings: &Settings, palette: &Palette, cx: &App) -> Div {
    let host = model.read(cx);
    let choice = host.choice(cx);
    let mut rows = vec![choice_row(model, choice, palette)];
    if choice == EngineChoice::Captain {
        rows.push(status_row(model, host, palette));
        rows.extend(engine_resources::rows(model, host, palette));
        rows.push(quit_row(settings, palette));
        rows.push(migrate_row(palette));
        rows.push(reset_row(model, host, palette));
    }
    settings_card("Captain Engine", rows, palette)
}

fn choice_row(model: &Entity<HostModel>, choice: EngineChoice, palette: &Palette) -> AnyElement {
    let segments = EngineChoice::ALL
        .into_iter()
        .map(|option| {
            let model = model.clone();
            Segment {
                label: option.label().into(),
                selected: option == choice,
                help: match option {
                    EngineChoice::Captain => {
                        "Use Captain Engine, the virtual machine that Captain runs for you."
                    }
                    EngineChoice::External => {
                        "Use another Docker engine. Captain connects to it but does not start or stop it."
                    }
                }
                .into(),
                on_click: Box::new(move |_, cx| {
                    model.update(cx, |model, cx| match option {
                        EngineChoice::Captain => model.use_captain(cx),
                        EngineChoice::External => model.use_external(cx),
                    })
                }),
            }
        })
        .collect();
    settings_row(
        "Engine",
        Some(
            "Captain Engine is a VM that Captain runs. Another engine is only connected to.".into(),
        ),
        segmented("engine-choice", segments, palette),
        palette,
    )
    .id("settings-engine-choice")
    .help("Choose the engine that Captain uses.")
    .into_any_element()
}

fn status_row(model: &Entity<HostModel>, host: &HostModel, palette: &Palette) -> AnyElement {
    let status = host.status().clone();
    let color = palette.host_status(&status);
    let note = match &status {
        HostStatus::NotInstalled(reason) | HostStatus::Failed(reason) => reason.clone(),
        HostStatus::NotCreated => "Not set up yet. Start sets it up.".into(),
        _ => host.resources().summary(),
    };
    let control = host.can_control();
    let button = |id: &'static str,
                  label: &'static str,
                  enabled: bool,
                  action: fn(&mut HostModel, &mut Context<HostModel>)| {
        let model = model.clone();
        text_button(
            id,
            label,
            ButtonTone::Accent,
            control && enabled,
            palette,
            move |_, _, cx| model.update(cx, action),
        )
    };
    let controls = div()
        .flex()
        .items_center()
        .gap(px(8.))
        .child(pill(
            status.label(),
            palette.readable(color),
            palette.tint(color),
        ))
        .child(button(
            "engine-start",
            "Start",
            status.can_start(),
            |model, cx| model.start(cx),
        ))
        .child(button(
            "engine-stop",
            "Stop",
            status.can_stop(),
            |model, cx| model.stop(cx).detach(),
        ))
        .child(button(
            "engine-restart",
            "Restart",
            status.is_running(),
            |model, cx| model.restart(cx),
        ));
    settings_row("Status", Some(note.into()), controls, palette)
        .id("settings-engine-status")
        .help("Start, stop, or restart Captain Engine. Its containers stop and start with it.")
        .into_any_element()
}

fn quit_row(settings: &Settings, palette: &Palette) -> AnyElement {
    settings_row(
        "Stop the engine when Captain quits",
        Some("Closing the window keeps Captain and the engine running.".into()),
        Switch::new("engine-stop-on-quit")
            .checked(settings.stop_engine_on_quit)
            .on_click(|checked, _, cx| {
                let checked = *checked;
                store::update(cx, |settings| settings.stop_engine_on_quit = checked);
            }),
        palette,
    )
    .id("settings-stop-on-quit")
    .help("Stop Captain Engine when you quit Captain. Its containers stop with it.")
    .into_any_element()
}

fn migrate_row(palette: &Palette) -> AnyElement {
    settings_row(
        "Bring data from another engine",
        Some("Copies volumes, images, and projects. The other engine stays as it is.".into()),
        text_button(
            "engine-migrate",
            "Bring data from another engine\u{2026}",
            ButtonTone::Accent,
            true,
            palette,
            |_, window, cx| window.dispatch_action(Box::new(OpenMigrationAssistant), cx),
        ),
        palette,
    )
    .id("settings-migrate").help("Open the Migration Assistant, which copies volumes, images, and projects from another engine.").into_any_element()
}

fn reset_row(model: &Entity<HostModel>, host: &HostModel, palette: &Palette) -> AnyElement {
    let model = model.clone();
    let enabled = host.can_control() && !host.status().is_busy();
    settings_row(
        "Reset",
        Some("Deletes the VM with all its containers, images, and volumes.".into()),
        text_button(
            "engine-reset",
            "Reset Captain Engine\u{2026}",
            ButtonTone::Danger,
            enabled,
            palette,
            move |_, window, cx| reset_dialog::open(model.clone(), window, cx),
        ),
        palette,
    )
    .id("settings-engine-reset")
    .help("Delete Captain Engine with all its containers, images, and volumes. Captain asks first.")
    .into_any_element()
}
