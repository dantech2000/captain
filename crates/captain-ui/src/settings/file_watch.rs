//! Watches `settings.json` and applies the edits made in an editor or by
//! `captain set`. A file with a mistake keeps the last good settings, and the
//! mistake shows as a toast and on the Diagnostics page. See ADR 0013.

use std::path::PathBuf;
use std::time::Duration;

use captain_core::settings::{FileProblem, LoadedSettings};
use gpui_kit::component::WindowExt;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;

use super::store;
use crate::diagnostics::diagnostics_model;
use crate::engine_host::host_model;

/// How often Captain reads the file. A read of a small file is cheap, and a poll
/// works the same on every OS, through a symlink, and with editors that replace
/// the file on save.
const POLL: Duration = Duration::from_secs(2);

/// A new mistake in the file, for a toast: "Captain keeps the last good settings.
/// settings.json line 9: …".
pub struct SettingsFileProblem(pub SharedString);

/// The last text of the file that Captain read, and its mistake, if any.
pub struct SettingsFileWatch {
    last_text: Option<String>,
    problem: Option<FileProblem>,
    _poll: Task<()>,
}

impl EventEmitter<SettingsFileProblem> for SettingsFileWatch {}

struct WatchHandle(Entity<SettingsFileWatch>);

impl Global for WatchHandle {}

/// Starts watching the settings file. Call it after [`store::init`].
pub fn init(cx: &mut App) {
    let Some(path) = store::settings_file_path(cx) else {
        return;
    };
    let watch = cx.new(|cx: &mut Context<SettingsFileWatch>| SettingsFileWatch {
        last_text: None,
        problem: None,
        _poll: cx.spawn(async move |this, cx| {
            loop {
                let text = read(path.clone(), cx).await;
                if this.update(cx, |watch, cx| watch.check(text, cx)).is_err() {
                    break;
                }
                cx.background_executor().timer(POLL).await;
            }
        }),
    });
    cx.set_global(WatchHandle(watch));
}

pub fn file_watch(cx: &App) -> Option<Entity<SettingsFileWatch>> {
    cx.try_global::<WatchHandle>()
        .map(|handle| handle.0.clone())
}

/// Shows each new mistake in the file as a toast in `window`.
pub fn toast_file_problems<V: 'static>(
    window: &mut Window,
    cx: &mut Context<V>,
) -> Option<Subscription> {
    let watch = file_watch(cx)?;
    Some(cx.subscribe_in(
        &watch,
        window,
        |_, _, problem: &SettingsFileProblem, window, cx| {
            window.push_notification(Notification::warning(problem.0.clone()), cx);
        },
    ))
}

/// The mistake that keeps Captain on the last good settings, for Diagnostics.
pub fn file_problem(cx: &App) -> Option<String> {
    let watch = file_watch(cx)?;
    watch.read(cx).problem.as_ref().map(ToString::to_string)
}

/// The file's text. A missing file reads as empty, which gives the defaults.
async fn read(path: PathBuf, cx: &mut AsyncApp) -> Option<String> {
    cx.background_executor()
        .spawn(async move {
            match std::fs::read_to_string(&path) {
                Ok(text) => Some(text),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Some(String::new()),
                Err(error) => {
                    tracing::warn!(%error, "cannot read {}", path.display());
                    None
                }
            }
        })
        .await
}

impl SettingsFileWatch {
    fn check(&mut self, text: Option<String>, cx: &mut Context<Self>) {
        let Some(text) = text.filter(|text| self.last_text.as_ref() != Some(text)) else {
            return;
        };
        self.last_text = Some(text.clone());
        let problem = match LoadedSettings::parse(text) {
            Err(problem) => Some(problem),
            Ok(loaded) => match loaded.problems.into_iter().next() {
                Some(problem) => Some(problem),
                None => {
                    apply(loaded.settings, cx);
                    None
                }
            },
        };
        if problem != self.problem {
            if let Some(problem) = &problem {
                tracing::warn!(%problem, "keeping the last good settings");
                let message = format!("Captain keeps the last good settings. {problem}");
                cx.emit(SettingsFileProblem(message.into()));
            }
            self.problem = problem;
            // Diagnostics observes this and runs its checks again.
            cx.notify();
        }
    }
}

/// Takes the file's settings, if they differ, and hands them to what follows them
/// outside the settings store: the engine and the log level.
fn apply(settings: captain_core::settings::Settings, cx: &mut App) {
    let before = store::current(cx);
    if settings == before {
        return;
    }
    tracing::info!("settings.json changed; applying it");
    store::replace(cx, settings.clone());
    if let Some(host) = host_model(cx) {
        host.update(cx, |host, cx| host.follow_file(&before, &settings, cx));
    }
    if before.debug_logging != settings.debug_logging
        && let Some(model) = diagnostics_model(cx)
    {
        model.read(cx).set_debug_logging(settings.debug_logging);
    }
}
