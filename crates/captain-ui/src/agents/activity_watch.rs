//! Watches `~/.captain/agent-activity.jsonl`, which `captain mcp` writes, for the
//! status bar and the Agent activity list. New actions also go to Captain's log.
//! See docs/features/0038-agent-tools.md.

use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use captain_core::agent_clients::client_label;
use captain_core::agent_tools::{Activity, activity_path, read_activity};
use gpui_kit::*;

/// How often Captain looks at the file. A look reads only its size and time.
const POLL: Duration = Duration::from_secs(2);
/// How many entries the list keeps.
const KEPT: usize = 50;
/// How long an entry shows in the status bar, in seconds.
const RECENT: i64 = 10 * 60;

/// The newest entries of the activity log, newest first.
pub struct AgentActivity {
    entries: Vec<Activity>,
    /// The file's size and change time at the last read.
    stamp: Option<(u64, SystemTime)>,
    _poll: Task<()>,
}

struct Handle(Entity<AgentActivity>);

impl Global for Handle {}

/// Starts watching the activity log.
pub fn init(cx: &mut App) {
    let Some(path) = std::env::home_dir().map(|home| activity_path(&home)) else {
        return;
    };
    let watch = cx.new(|cx: &mut Context<AgentActivity>| AgentActivity {
        entries: Vec::new(),
        stamp: None,
        _poll: cx.spawn(async move |this, cx| {
            loop {
                let stamp = stamp(&path);
                let changed = this
                    .read_with(cx, |watch, _| watch.stamp != stamp)
                    .unwrap_or(false);
                if changed {
                    let read = path.clone();
                    let entries = cx
                        .background_executor()
                        .spawn(async move { read_activity(&read, KEPT) })
                        .await;
                    if this
                        .update(cx, |watch, cx| watch.take(stamp, entries, cx))
                        .is_err()
                    {
                        break;
                    }
                }
                cx.background_executor().timer(POLL).await;
            }
        }),
    });
    cx.set_global(Handle(watch));
}

pub fn agent_activity(cx: &App) -> Option<Entity<AgentActivity>> {
    cx.try_global::<Handle>().map(|handle| handle.0.clone())
}

/// The newest entry of the last ten minutes, for the status bar.
pub fn latest_activity(cx: &App) -> Option<Activity> {
    let watch = agent_activity(cx)?;
    let latest = watch.read(cx).entries.first()?;
    let now = chrono::Utc::now().timestamp();
    (now - latest.at <= RECENT).then(|| latest.clone())
}

fn stamp(path: &PathBuf) -> Option<(u64, SystemTime)> {
    let meta = std::fs::metadata(path).ok()?;
    Some((meta.len(), meta.modified().ok()?))
}

impl AgentActivity {
    pub fn entries(&self) -> &[Activity] {
        &self.entries
    }

    fn take(
        &mut self,
        stamp: Option<(u64, SystemTime)>,
        entries: Vec<Activity>,
        cx: &mut Context<Self>,
    ) {
        // The first read only fills the list; later ones log what is new.
        if self.stamp.is_some() {
            let newest = self.entries.first().map_or(i64::MIN, |entry| entry.at);
            for entry in entries.iter().rev().filter(|e| e.at >= newest) {
                if entry.is_action() && !self.entries.contains(entry) {
                    tracing::info!(
                        client = %client_label(&entry.client),
                        call = %entry.summary(),
                        ok = entry.ok,
                        result = %entry.result,
                        "agent action"
                    );
                }
            }
        }
        self.stamp = stamp;
        self.entries = entries;
        cx.notify();
    }
}
