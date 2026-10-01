use captain_core::model::RestartPolicy;
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::*;

use super::form_state::RunImage;
use crate::help::HelpExt;
use crate::theme::Palette;
use crate::widgets::{Segment, segmented};

impl RunImage {
    pub(super) fn restart_control(&self, palette: &Palette, cx: &mut Context<Self>) -> Div {
        let segments = RestartPolicy::ALL
            .into_iter()
            .map(|policy| {
                let this = cx.entity().downgrade();
                Segment {
                    label: policy.name().into(),
                    selected: self.restart == policy,
                    help: match policy {
                        RestartPolicy::No => "Never restart the container.",
                        RestartPolicy::UnlessStopped => {
                            "Restart the container when it exits, unless you stopped it."
                        }
                        RestartPolicy::Always => {
                            "Always restart the container, also after the engine starts."
                        }
                        RestartPolicy::OnFailure => {
                            "Restart the container only when it exits with an error."
                        }
                    }
                    .into(),
                    on_click: Box::new(move |_, cx| {
                        this.update(cx, |this, cx| this.set_restart(policy, cx))
                            .ok();
                    }),
                }
            })
            .collect();
        div()
            .flex()
            .child(segmented("restart-policy", segments, palette))
    }

    pub(super) fn auto_remove_control(&self, cx: &mut Context<Self>) -> Stateful<Div> {
        let this = cx.entity().downgrade();
        div()
            .id("run-auto-remove")
            .child(
                Checkbox::new("auto-remove")
                    .label("Remove the container when it exits")
                    .checked(self.auto_remove)
                    .on_click(move |checked, _, cx| {
                        this.update(cx, |this, cx| this.set_auto_remove(*checked, cx))
                            .ok();
                    }),
            )
            .help("Delete the container as soon as it stops, like docker run --rm.")
    }
}
