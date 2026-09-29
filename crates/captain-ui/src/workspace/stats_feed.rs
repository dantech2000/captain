use captain_core::model::ContainerState;
use futures::StreamExt;
use gpui_kit::*;

use super::Workspace;

impl Workspace {
    /// Follows stats for every running container, and stops following the rest.
    pub(super) fn sync_stats(&mut self, cx: &mut Context<Self>) {
        let Some(engine) = self.engine.clone() else {
            return;
        };
        let running: Vec<String> = self
            .store
            .containers()
            .iter()
            .filter(|c| c.state == ContainerState::Running)
            .map(|c| c.id.clone())
            .collect();

        self.stats_tasks.retain(|id, _| running.contains(id));
        self.stats.retain(|id| running.iter().any(|r| r == id));

        for id in running {
            if self.stats_tasks.contains_key(&id) {
                continue;
            }
            let mut samples = engine.stats(&id);
            let key = id.clone();
            let task = cx.spawn(async move |this, cx| {
                while let Some(sample) = samples.next().await {
                    let Ok(sample) = sample else { break };
                    let pushed = this.update(cx, |this, cx| {
                        this.stats.push(&key, sample);
                        cx.notify();
                    });
                    if pushed.is_err() {
                        break;
                    }
                }
            });
            self.stats_tasks.insert(id, task);
        }
    }
}
