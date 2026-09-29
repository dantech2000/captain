//! Quit: stops Captain Engine first when the settings say so. See ADR 0008.

use std::time::Duration;

use futures::future;
use gpui_kit::*;

/// How long Quit waits for the engine to stop before Captain exits anyway.
const STOP_WAIT: Duration = Duration::from_secs(20);

/// Set while Quit waits for the engine. A second Quit exits at once.
struct Quitting;

impl Global for Quitting {}

/// Stops Captain Engine if it runs and the settings ask for it, then quits. The
/// window stays responsive and shows "Stopping" meanwhile.
pub fn quit(cx: &mut App) {
    if cx.has_global::<Quitting>() {
        cx.quit();
        return;
    }
    let Some(model) = captain_ui::host_model(cx) else {
        cx.quit();
        return;
    };
    let host = model.read(cx);
    let stop_engine = captain_ui::current_settings(cx).stop_engine_on_quit
        && host.uses_captain(cx)
        && host.can_control()
        && host.status().can_stop();
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
