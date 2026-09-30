use captain_core::model::{ContainerDetail, RestartPolicy};
use captain_core::project_map::{Setting, StagedChange};
use gpui_kit::*;

use super::map::Draft;
use super::view_tabs::ProjectTab;
use super::{ProjectNotice, ProjectView};

/// The current value of each setting the map can stage.
pub fn current_settings(detail: &ContainerDetail) -> [Setting; 3] {
    [
        Setting::Memory(detail.memory_limit),
        Setting::Cpus(detail.nano_cpus),
        Setting::Restart(RestartPolicy::parse(&detail.restart_policy)),
    ]
}

impl ProjectView {
    /// Shows Overview or Map. The map reads the volume sizes each time it opens.
    pub(super) fn show_tab(&mut self, tab: ProjectTab, cx: &mut Context<Self>) {
        self.tab = tab;
        if tab == ProjectTab::Map {
            self.load_volumes(cx);
        }
        cx.notify();
    }

    /// Stages `to` for container `id`, from its inspected value.
    pub(super) fn stage(&mut self, id: &str, name: &str, to: Setting, cx: &mut Context<Self>) {
        let Some((_, detail)) = self.details.get(id) else {
            return;
        };
        let Some(from) = current_settings(detail)
            .into_iter()
            .find(|s| s.same_field(&to))
        else {
            return;
        };
        self.staged.stage(StagedChange {
            container_id: id.to_string(),
            container: name.to_string(),
            from,
            to,
        });
        cx.notify();
    }

    pub(super) fn unstage(&mut self, id: &str, field: Setting, cx: &mut Context<Self>) {
        self.staged.remove(id, &field);
        cx.notify();
    }

    /// Drops the staged changes of the containers `ids`.
    pub(super) fn discard(&mut self, ids: &[String], cx: &mut Context<Self>) {
        self.staged
            .retain_containers(|id| !ids.iter().any(|shown| shown == id));
        cx.notify();
    }

    /// Opens the editor on a node, with the staged values where there are any.
    pub(super) fn edit(&mut self, id: &str, name: &str, cx: &mut Context<Self>) {
        let Some((_, detail)) = self.details.get(id) else {
            return;
        };
        let value = |current: Setting| self.staged.staged(id, &current).unwrap_or(current);
        let [memory, cpus, restart] = current_settings(detail).map(value);
        let (Setting::Memory(memory), Setting::Cpus(nano_cpus), Setting::Restart(restart)) =
            (memory, cpus, restart)
        else {
            return;
        };
        self.map.draft = Some(Draft {
            id: id.to_string(),
            name: name.to_string(),
            memory,
            nano_cpus,
            restart,
        });
        cx.notify();
    }

    /// Stages the editor's values and closes it.
    pub(super) fn stage_draft(&mut self, cx: &mut Context<Self>) {
        let Some(draft) = self.map.draft.take() else {
            return;
        };
        for to in [
            Setting::Memory(draft.memory),
            Setting::Cpus(draft.nano_cpus),
            Setting::Restart(draft.restart),
        ] {
            self.stage(&draft.id, &draft.name, to, cx);
        }
        cx.notify();
    }

    /// Sends the staged changes of the containers `ids`, one update per container.
    /// Applied rows leave; failed ones stay.
    pub(super) fn apply(&mut self, ids: &[String], cx: &mut Context<Self>) {
        let Some(engine) = self.workspace.read(cx).engine() else {
            return;
        };
        let plans = self.staged.plan();
        for plan in plans.into_iter().filter(|p| ids.contains(&p.container_id)) {
            if !self.map.applying.insert(plan.container_id.clone()) {
                continue;
            }
            let summary: Vec<String> = self
                .staged
                .changes()
                .iter()
                .filter(|c| c.container_id == plan.container_id)
                .map(|c| format!("{} {}", c.to.field().to_lowercase(), c.to.value_label()))
                .collect();
            let update = engine.update_resources(&plan.container_id, plan.update);
            cx.spawn(async move |this, cx| {
                let result = update.await;
                this.update(cx, |this, cx| {
                    let id = &plan.container_id;
                    this.map.applying.remove(id);
                    let notice = match result {
                        Ok(()) => {
                            this.staged.retain_containers(|staged| staged != id);
                            this.details.remove(id);
                            ProjectNotice::Updated {
                                name: plan.container.clone(),
                                summary: summary.join(", "),
                            }
                        }
                        Err(error) => ProjectNotice::Failed {
                            title: format!("Cannot update {}", plan.container),
                            error: error.to_string(),
                        },
                    };
                    cx.emit(notice);
                    this.follow(cx);
                    cx.notify();
                })
                .ok();
            })
            .detach();
        }
        cx.notify();
    }

    /// Reads the volume list for the sizes on the map's volume nodes.
    pub(super) fn load_volumes(&mut self, cx: &mut Context<Self>) {
        let Some(engine) = self.workspace.read(cx).engine() else {
            return;
        };
        let list = engine.list_volumes();
        self.map.volumes_task = Some(cx.spawn(async move |this, cx| {
            let Ok(volumes) = list.await else { return };
            this.update(cx, |this, cx| {
                this.map.volume_sizes = volumes
                    .into_iter()
                    .filter_map(|v| Some((v.name, v.size_bytes?)))
                    .collect();
                cx.notify();
            })
            .ok();
        }));
    }
}
