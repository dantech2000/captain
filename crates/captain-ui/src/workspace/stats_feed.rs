use std::time::Duration;

use captain_core::model::ContainerState;
use futures::StreamExt;
use gpui_kit::*;

use super::Workspace;

/// The wait before following stats again after the stream ends. It doubles after
/// each end without a sample.
const FIRST_RETRY: Duration = Duration::from_secs(1);
const MAX_RETRY: Duration = Duration::from_secs(30);

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
            self.stats_generation += 1;
            let generation = self.stats_generation;
            let engine = engine.clone();
            let key = id.clone();
            // A stream that fails or ends starts again while the container runs.
            let task = cx.spawn(async move |this, cx| {
                let mut delay = FIRST_RETRY;
                loop {
                    let mut samples = engine.stats(&key);
                    while let Some(Ok(sample)) = samples.next().await {
                        delay = FIRST_RETRY;
                        let pushed = this.update(cx, |this, cx| {
                            this.stats.push(&key, sample);
                            cx.notify();
                        });
                        if pushed.is_err() {
                            return;
                        }
                    }
                    drop(samples);
                    cx.background_executor().timer(delay).await;
                    delay = (delay * 2).min(MAX_RETRY);
                    let running = this.update(cx, |this, _| {
                        let running = this
                            .store
                            .find(&key)
                            .is_some_and(|c| c.state == ContainerState::Running);
                        if !running {
                            this.finish_stats(&key, generation);
                        }
                        running
                    });
                    if !matches!(running, Ok(true)) {
                        return;
                    }
                }
            });
            self.stats_tasks.insert(id, (generation, task));
        }
    }

    /// Forgets the stats task of `id` if it is still the one numbered `generation`.
    /// The task calls this itself, so it detaches rather than cancels.
    fn finish_stats(&mut self, id: &str, generation: u64) {
        if self
            .stats_tasks
            .get(id)
            .is_some_and(|(current, _)| *current == generation)
            && let Some((_, task)) = self.stats_tasks.remove(id)
        {
            task.detach();
        }
    }
}
