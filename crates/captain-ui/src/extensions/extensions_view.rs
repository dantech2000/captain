use gpui_kit::component::Sizable;
use gpui_kit::component::WindowExt;
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::spinner::Spinner;
use gpui_kit::*;

use super::{ExtensionEvent, ExtensionsModel, extension_row, install_dialog};
use crate::help::HelpExt;
use crate::icons::CaptainIcon;
use crate::theme::Palette;
use crate::widgets::{
    ButtonTone, empty_note, inline_error, page_header, settings_card, text_button,
};
use crate::workspace::{Page, Workspace};

/// The Extensions page: the trust note, the install field, and the installed list.
pub struct ExtensionsView {
    model: Entity<ExtensionsModel>,
    reference: Entity<InputState>,
    /// True while the page shows, to read the folder again when it opens.
    showing: bool,
    _subscriptions: Vec<Subscription>,
}

impl ExtensionsView {
    pub fn new(workspace: Entity<Workspace>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let model = cx.new(|cx| ExtensionsModel::new(workspace.clone(), cx));
        let reference = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Image, for example docker/disk-usage-extension")
        });
        let subscriptions = vec![
            cx.observe(&model, |_, _, cx| cx.notify()),
            cx.subscribe_in(
                &model,
                window,
                |this, _, event: &ExtensionEvent, window, cx| {
                    if let ExtensionEvent::Confirm(candidate) = event {
                        install_dialog::open(this.model.clone(), (**candidate).clone(), window, cx);
                    }
                    if let ExtensionEvent::ConfirmUpdate(update) = event {
                        install_dialog::open_update(
                            this.model.clone(),
                            (**update).clone(),
                            window,
                            cx,
                        );
                    }
                    if let ExtensionEvent::Installed(_) = event {
                        this.reference
                            .update(cx, |input, cx| input.set_value("", window, cx));
                    }
                    if let Some(notification) = event.notification() {
                        window.push_notification(notification, cx);
                    }
                },
            ),
            cx.subscribe_in(&reference, window, |this, _, event, _, cx| {
                if let InputEvent::PressEnter { .. } = event {
                    this.install(cx);
                }
            }),
            cx.observe(&workspace, |view: &mut Self, workspace, cx| {
                let showing = workspace.read(cx).page() == Page::Extensions;
                if showing && !view.showing {
                    view.model.update(cx, |model, cx| model.reload(cx));
                }
                view.showing = showing;
            }),
        ];
        Self {
            model,
            reference,
            showing: false,
            _subscriptions: subscriptions,
        }
    }

    fn install(&mut self, cx: &mut Context<Self>) {
        let reference = self.reference.read(cx).value().trim().to_string();
        if !reference.is_empty() {
            self.model
                .update(cx, |model, cx| model.prepare(reference, cx));
        }
    }
}

/// "2 extensions", or the step that runs now.
fn summary(model: &ExtensionsModel) -> SharedString {
    if let Some(step) = model.step() {
        return step;
    }
    match model.list().len() {
        1 => "1 extension".into(),
        count => format!("{count} extensions").into(),
    }
}

impl Render for ExtensionsView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let model = self.model.read(cx);
        let connected = model.manager(cx).is_some();
        let enabled = connected && !model.is_busy();
        let this = cx.entity().downgrade();
        let field = div()
            .flex()
            .items_center()
            .gap(px(8.))
            .child(div().flex_1().child(Input::new(&self.reference).small()))
            .child(
                text_button(
                    "extension-install",
                    "Install",
                    ButtonTone::Accent,
                    enabled,
                    &palette,
                    move |_, _, cx| {
                        this.update(cx, |view, cx| view.install(cx)).ok();
                    },
                )
                .help("Install the extension image named in the field. Captain asks first."),
            );
        let install = div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .px(px(14.))
            .py(px(12.))
            .child(field)
            .child(
                div()
                    .text_size(px(12.))
                    .text_color(palette.warn_text)
                    .child(install_dialog::TRUST_NOTE),
            )
            .children((!super::CAN_OPEN).then(|| {
                inline_error("Extension windows need macOS. Captain can install extensions here, but not open them.", &palette)
            }))
            .into_any_element();

        let list: AnyElement = if !connected {
            inline_error("Extensions need a connected engine.", &palette).into_any_element()
        } else if model.is_loaded() && model.list().is_empty() {
            empty_note(
                CaptainIcon::Extension,
                "No extensions",
                "Install a Docker Desktop extension by its image reference.",
                &palette,
            )
            .into_any_element()
        } else {
            let rows = model
                .list()
                .iter()
                .map(|extension| extension_row::render(&self.model, extension, enabled, &palette));
            settings_card("Installed", rows, &palette).into_any_element()
        };
        let trailing = model
            .is_busy()
            .then(|| Spinner::new().color(palette.text2).into_any_element());

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(page_header(
                "extensions-header",
                "Extensions",
                summary(model),
                trailing,
                &palette,
            ))
            .child(
                div()
                    .id("extensions-scroll")
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
                            .gap(px(20.))
                            .child(settings_card("Install", [install], &palette))
                            .child(list),
                    ),
            )
    }
}
