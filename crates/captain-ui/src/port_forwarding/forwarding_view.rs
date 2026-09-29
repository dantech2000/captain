use gpui_kit::assets::IconName;
use gpui_kit::component::WindowExt;
use gpui_kit::*;

use super::{ForwardingModel, service_card};
use crate::kubernetes::KubeEvent;
use crate::theme::Palette;
use crate::widgets::{empty_note, inline_error, page_header};
use crate::workspace::{Page, Workspace};

/// The Port Forwarding page. It reads the Services only while it shows.
pub struct PortForwardingView {
    model: Entity<ForwardingModel>,
    _subscriptions: Vec<Subscription>,
}

impl PortForwardingView {
    pub fn new(workspace: Entity<Workspace>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let model = cx.new(ForwardingModel::new);
        let subscriptions = vec![
            cx.observe(&model, |_, _, cx| cx.notify()),
            cx.subscribe_in(&model, window, |_, _, event: &KubeEvent, window, cx| {
                window.push_notification(event.notification(), cx);
            }),
            cx.observe(&workspace, |view: &mut Self, workspace, cx| {
                let showing = workspace.read(cx).page() == Page::PortForwarding;
                view.model
                    .update(cx, |model, cx| model.set_showing(showing, cx));
            }),
        ];
        Self {
            model,
            _subscriptions: subscriptions,
        }
    }
}

/// "4 services · 1 forwarded".
fn summary(model: &ForwardingModel) -> SharedString {
    let count = model.services().len();
    let noun = if count == 1 { "service" } else { "services" };
    format!("{count} {noun} · {} forwarded", model.forward_count()).into()
}

impl Render for PortForwardingView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let model = self.model.read(cx);
        let mut body: Vec<AnyElement> = Vec::new();
        if let Some(error) = model.error() {
            body.push(inline_error(error, &palette).into_any_element());
        } else if model.is_loaded() && model.services().is_empty() {
            body.push(
                empty_note(
                    IconName::ArrowRightLeft,
                    "No services",
                    "Services with TCP ports show here once you create them.",
                    &palette,
                )
                .into_any_element(),
            );
        }
        body.extend(service_card::render(
            &self.model,
            model,
            model.services(),
            &palette,
        ));

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(page_header(
                "forwarding-header",
                "Port Forwarding",
                summary(model),
                None,
                &palette,
            ))
            .child(
                div()
                    .id("forwarding-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .px(px(24.))
                    .pt(px(4.))
                    .pb(px(24.))
                    .child(
                        div()
                            .w_full()
                            .max_w(px(720.))
                            .flex()
                            .flex_col()
                            .gap(px(22.))
                            .children(body),
                    ),
            )
    }
}
