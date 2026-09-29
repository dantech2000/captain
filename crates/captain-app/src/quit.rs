//! Quit: stops Captain Engine first when the settings say so, or when a start runs.
//! It waits for a snapshot step. See ADR 0008 and docs/features/0023-snapshots.md.

use std::time::Duration;

use futures::future;
use gpui_kit::*;

/// How long Quit waits for the engine to stop before Captain exits anyway.
const STOP_WAIT: Duration = Duration::from_secs(20);

/// Set while Quit waits for the engine. A second Quit exits at once.
struct Quitting;

impl Global for Quitting {}

/// Set while Quit waits for a snapshot step to end.
struct WaitingForSnapshot {
    _subscription: Subscription,
}

impl Global for WaitingForSnapshot {}

/// Stops Captain Engine if it runs and the settings ask for it, then quits. A start
/// that runs now is stopped too, because Captain cannot finish it after it exits.
/// The window stays responsive and shows "Stopping" meanwhile.
pub fn quit(cx: &mut App) {
    let Some(model) = captain_ui::host_model(cx) else {
        cx.quit();
        return;
    };
    if model.read(cx).is_snapshotting() {
        wait_for_snapshot(model, cx);
        return;
    }
    if cx.has_global::<Quitting>() {
        cx.quit();
        return;
    }
    let host = model.read(cx);
    let stop_engine = host.uses_captain(cx)
        && host.can_control()
        && (host.is_starting()
            || captain_ui::current_settings(cx).stop_engine_on_quit && host.status().can_stop());
    if !stop_engine {
        cx.quit();
        return;
    }
    tracing::info!("stopping Captain Engine before quitting");
    cx.set_global(Quitting);
    let stop = model.update(cx, |model, cx| model.stop(cx));
    cx.spawn(async move |cx| {
        let timeout = cx.background_executor().timer(STOP_WAIT);
        if let future::Either::Right(_) = future::select(stop, Box::pin(timeout)).await {
            tracing::warn!("Captain Engine did not stop in time; quitting anyway");
        }
        cx.update(|cx| cx.quit());
    })
    .detach();
}

/// Quits when the snapshot step ends, because a step cut off halfway can leave the
/// engine without its disk. The step does not start the engine again.
fn wait_for_snapshot(model: Entity<captain_ui::HostModel>, cx: &mut App) {
    if cx.has_global::<WaitingForSnapshot>() {
        return;
    }
    tracing::info!("waiting for the snapshot step before quitting");
    model.update(cx, |model, cx| model.quit_after_snapshot(cx));
    let mut done = false;
    let subscription = cx.observe(&model, move |model, cx| {
        if done || model.read(cx).is_snapshotting() {
            return;
        }
        done = true;
        quit(cx);
    });
    cx.set_global(WaitingForSnapshot {
        _subscription: subscription,
    });
}
