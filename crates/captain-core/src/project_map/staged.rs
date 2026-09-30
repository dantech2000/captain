use std::mem::discriminant;

use crate::format::bytes_label;
use crate::model::{ResourceUpdate, RestartPolicy};

/// A setting that `docker update` changes in place, with its value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Setting {
    /// The memory limit in bytes. 0 means no limit.
    Memory(u64),
    /// The CPU limit in billionths of a CPU. 0 means no limit.
    Cpus(u64),
    Restart(RestartPolicy),
}

impl Setting {
    /// The field's name, for example `Memory limit`.
    pub fn field(&self) -> &'static str {
        match self {
            Self::Memory(_) => "Memory limit",
            Self::Cpus(_) => "CPUs",
            Self::Restart(_) => "Restart policy",
        }
    }

    /// True if both are values of the same field.
    pub fn same_field(&self, other: &Setting) -> bool {
        discriminant(self) == discriminant(other)
    }

    /// The value, for example `512 MB`, `1.5`, `unless-stopped`, or `No limit`.
    pub fn value_label(&self) -> String {
        match *self {
            Self::Memory(0) | Self::Cpus(0) => "No limit".into(),
            Self::Memory(bytes) => bytes_label(bytes),
            Self::Cpus(nano) => {
                let cpus = nano as f64 / 1e9;
                let label = format!("{cpus:.2}");
                label
                    .trim_end_matches('0')
                    .trim_end_matches('.')
                    .to_string()
            }
            Self::Restart(policy) => policy.name().into(),
        }
    }
}

/// One field of one container, from its current value to the staged one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StagedChange {
    pub container_id: String,
    /// The name the map shows for the container.
    pub container: String,
    pub from: Setting,
    pub to: Setting,
}

/// The updates Apply sends to one container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdatePlan {
    pub container_id: String,
    pub container: String,
    pub update: ResourceUpdate,
}

/// Changes that wait until Apply, in the order they were first staged.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StagedChanges {
    changes: Vec<StagedChange>,
}

impl StagedChanges {
    pub fn changes(&self) -> &[StagedChange] {
        &self.changes
    }

    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    /// Stages `change`. A change to a field that is staged already replaces the new
    /// value and keeps the old one; going back to the old value removes the row.
    pub fn stage(&mut self, change: StagedChange) {
        let at = self.position(&change.container_id, &change.to);
        match at {
            Some(at) if self.changes[at].from == change.to => {
                self.changes.remove(at);
            }
            Some(at) => self.changes[at].to = change.to,
            None if change.from == change.to => {}
            None => self.changes.push(change),
        }
    }

    /// Removes the staged change to `field`'s field of container `id`.
    pub fn remove(&mut self, id: &str, field: &Setting) {
        if let Some(at) = self.position(id, field) {
            self.changes.remove(at);
        }
    }

    pub fn clear(&mut self) {
        self.changes.clear();
    }

    /// Keeps only the changes of containers for which `keep` is true.
    pub fn retain_containers(&mut self, keep: impl Fn(&str) -> bool) {
        self.changes.retain(|change| keep(&change.container_id));
    }

    /// The staged value of `field`'s field of container `id`.
    pub fn staged(&self, id: &str, field: &Setting) -> Option<Setting> {
        self.position(id, field).map(|at| self.changes[at].to)
    }

    /// True if container `id` has a staged change.
    pub fn has(&self, id: &str) -> bool {
        self.changes.iter().any(|change| change.container_id == id)
    }

    /// One update per container, in the order the containers were first staged.
    pub fn plan(&self) -> Vec<UpdatePlan> {
        let mut plans: Vec<UpdatePlan> = Vec::new();
        for change in &self.changes {
            let at = match plans
                .iter()
                .position(|plan| plan.container_id == change.container_id)
            {
                Some(at) => at,
                None => {
                    plans.push(UpdatePlan {
                        container_id: change.container_id.clone(),
                        container: change.container.clone(),
                        update: ResourceUpdate::default(),
                    });
                    plans.len() - 1
                }
            };
            let update = &mut plans[at].update;
            match change.to {
                Setting::Memory(bytes) => update.memory = Some(bytes),
                Setting::Cpus(nano) => update.nano_cpus = Some(nano),
                Setting::Restart(policy) => update.restart_policy = Some(policy),
            }
        }
        plans
    }

    fn position(&self, id: &str, field: &Setting) -> Option<usize> {
        self.changes
            .iter()
            .position(|change| change.container_id == id && change.to.same_field(field))
    }
}

#[cfg(test)]
mod tests;
