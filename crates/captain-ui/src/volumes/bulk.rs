//! Selecting several volumes and deleting them at once. See
//! docs/features/0018-bulk-selection.md.

use std::time::Duration;

use captain_core::model::{BulkOutcome, count_label};
use captain_core::store::SelectMode;
use futures::future::join_all;
use gpui_kit::component::WindowExt;
use gpui_kit::component::button::ButtonVariant;
use gpui_kit::*;

use super::VolumesView;

impl VolumesView {
    /// A click on a row, with the keys held. A plain click selects the volume alone,
    /// or closes the panel when it is the only selected one.
    pub(super) fn click_row(&mut self, name: String, mode: SelectMode, cx: &mut Context<Self>) {
        if mode == SelectMode::Replace {
            if self.checked.is_bulk() {
                // Leaves the bulk selection instead of closing the panel.
                self.selected = None;
            }
            self.select(name.clone(), cx);
            match &self.selected {
                Some(_) => self.checked.click(&name, mode, &[]),
                None => self.checked.clear(),
            }
            return;
        }
        let order: Vec<String> = self
            .store
            .groups(self.filter)
            .into_iter()
            .flat_map(|group| group.items.into_iter().map(|v| v.name))
            .collect();
        self.checked.click(&name, mode, &order);
        if self.checked.contains(&name) {
            self.selected = Some(name);
        } else if self.selected.as_deref() == Some(name.as_str()) {
            self.selected = self.checked.keys().last().cloned();
        }
        self.load_users(cx);
        cx.notify();
    }

    pub(super) fn clear_bulk(&mut self, cx: &mut Context<Self>) {
        self.checked.clear();
        if let Some(name) = &self.selected {
            self.checked.click(name, SelectMode::Replace, &[]);
        }
        cx.notify();
    }

    /// Asks once, listing the names, then deletes the selected volumes.
    pub(super) fn confirm_bulk_delete(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let names = self.checked.keys().to_vec();
        let generation = self.generation;
        let title = SharedString::from(format!("Delete {}?", count_label(names.len(), "volume")));
        let description = SharedString::from(format!(
            "{}\n\nThis deletes their data. The engine refuses a volume that a container uses.",
            names.join(", ")
        ));
        let view = cx.entity();
        window.open_alert_dialog(cx, move |alert, _, _| {
            let view = view.clone();
            let names = names.clone();
            alert
                .title(title.clone())
                .description(description.clone())
                .show_cancel(true)
                .ok_text("Delete")
                .ok_variant(ButtonVariant::Danger)
                .on_ok(move |_, _, cx| {
                    let names = names.clone();
                    view.update(cx, |view, cx| view.remove_many(names, generation, cx));
                    true
                })
        });
    }

    /// Removes each volume, then reports the result on the page: a notice, or an
    /// error line with each volume that failed. Does nothing after an engine switch
    /// since the confirmation opened.
    fn remove_many(&mut self, names: Vec<String>, generation: u64, cx: &mut Context<Self>) {
        let Some(engine) = self
            .engine
            .clone()
            .filter(|_| generation == self.generation)
        else {
            return;
        };
        let names: Vec<_> = names
            .into_iter()
            .filter(|name| self.removing.insert(name.clone()))
            .collect();
        self.error = None;
        self.notice = None;
        cx.notify();
        let runs = join_all(names.into_iter().map(|name| {
            let run = engine.remove_volume(&name);
            async move { (name, run.await) }
        }));
        cx.spawn(async move |this, cx| {
            let results = runs.await;
            this.update(cx, |this, cx| {
                let mut outcome = BulkOutcome::default();
                for (name, result) in results {
                    this.removing.remove(&name);
                    match result {
                        Ok(()) => outcome.done.push(name),
                        Err(error) => outcome.failed.push((name, error)),
                    }
                }
                if outcome.failed.is_empty() {
                    this.notice = Some(outcome.done_message("Deleted", "volume"));
                } else {
                    this.error = Some(format!(
                        "{}:\n{}",
                        outcome.failed_title("Delete", "volume"),
                        outcome.failed_lines()
                    ));
                }
                this.reload(Duration::ZERO, cx);
                cx.notify();
            })
            .ok();
        })
        .detach();
    }
}
