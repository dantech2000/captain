use captain_core::model::ImageDetail;
use gpui_kit::component::WindowExt;
use gpui_kit::*;

use super::host::SheetHost;
use super::keys::{CONTEXT, Confirm, Pick1, Pick2, Pick3, Pick4, SelectNext, SelectPrev};
use super::options::NewOption;
use super::paste::PasteStep;
use super::run_image::RunImage;
use super::sheet_view;
use super::template::TemplateStep;
use crate::images::ImagesState;
use crate::project::ProjectView;
use crate::workspace::Workspace;

/// What the sheet says under the cards while it opens a folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SheetStatus {
    Idle,
    /// Checking the Compose files with `docker compose config`.
    Checking(String),
    Failed(String),
}

/// What the sheet shows: the four cards, or the form of one of them.
pub enum Step {
    Choose,
    Template(Entity<TemplateStep>),
    RunImage(Entity<RunImage>),
    Paste(Entity<PasteStep>),
}

/// The New sheet: four ways to make a project. See
/// docs/features/0040-new-projects.md.
pub struct NewSheet {
    pub(super) host: SheetHost,
    /// The highlighted card, an index into [`NewOption::ALL`].
    pub(super) highlight: usize,
    pub(super) status: SheetStatus,
    pub(super) focus: FocusHandle,
    pub(super) check: Option<Task<()>>,
    pub(super) step: Step,
}

/// Opens the New sheet with the cards, Run an image highlighted. Does nothing
/// while a sheet or dialog is open, so ⌘N never stacks a second one.
pub fn open(
    workspace: Entity<Workspace>,
    project: Entity<ProjectView>,
    window: &mut Window,
    cx: &mut App,
) {
    if window.has_active_dialog(cx) {
        return;
    }
    let sheet = cx.new(|cx| NewSheet::new(SheetHost { workspace, project }, cx));
    show(sheet, window, cx);
}

/// Opens the New sheet at Run an image with `image` chosen, for the Images page's
/// Run. `images` shows the Started notice for a plain container.
pub fn open_run_image(
    host: SheetHost,
    image: String,
    detail: ImageDetail,
    images: Entity<ImagesState>,
    window: &mut Window,
    cx: &mut App,
) {
    if window.has_active_dialog(cx) {
        return;
    }
    let sheet = cx.new(|cx| NewSheet::new(host.clone(), cx));
    let weak = sheet.downgrade();
    let step = cx.new(|cx| {
        let mut step = RunImage::new(host, weak, Some(images), window, cx);
        step.choose_local(image, detail, window, cx);
        step
    });
    sheet.update(cx, |sheet, _| sheet.step = Step::RunImage(step));
    show(sheet, window, cx);
}

fn show(sheet: Entity<NewSheet>, window: &mut Window, cx: &mut App) {
    let focus = sheet.read(cx).focus.clone();
    let on_cards = matches!(sheet.read(cx).step, Step::Choose);
    window.open_dialog(cx, move |dialog, _, _| {
        dialog
            .w(px(580.))
            .margin_top(px(80.))
            .overlay_closable(false)
            .child(sheet.clone())
    });
    // The dialog takes focus as it opens; the sheet takes it back for its keys.
    if on_cards {
        window.defer(cx, move |window, cx| focus.focus(window, cx));
    }
}

impl NewSheet {
    fn new(host: SheetHost, cx: &mut Context<Self>) -> Self {
        Self {
            host,
            highlight: 0,
            status: SheetStatus::Idle,
            focus: cx.focus_handle(),
            check: None,
            step: Step::Choose,
        }
    }

    /// Opens the card `option`: its form, or the folder picker.
    pub(super) fn choose(
        &mut self,
        option: NewOption,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(index) = NewOption::ALL.iter().position(|o| *o == option) {
            self.highlight = index;
        }
        if matches!(self.status, SheetStatus::Checking(_)) {
            return;
        }
        self.status = SheetStatus::Idle;
        let (host, sheet) = (self.host.clone(), cx.entity().downgrade());
        self.step = match option {
            NewOption::OpenFolder => {
                self.pick_folder(window, cx);
                Step::Choose
            }
            NewOption::RunImage => {
                Step::RunImage(cx.new(|cx| RunImage::new(host, sheet, None, window, cx)))
            }
            NewOption::Template => {
                Step::Template(cx.new(|cx| TemplateStep::new(host, sheet, window, cx)))
            }
            NewOption::PasteRun => {
                Step::Paste(cx.new(|cx| PasteStep::new(host, sheet, window, cx)))
            }
        };
        cx.notify();
    }

    /// Goes back to the four cards.
    pub(super) fn back(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.step = Step::Choose;
        self.focus.focus(window, cx);
        cx.notify();
    }

    fn step(&mut self, by: isize, cx: &mut Context<Self>) {
        let count = NewOption::ALL.len() as isize;
        self.highlight = (self.highlight as isize + by).rem_euclid(count) as usize;
        cx.notify();
    }
}

impl Render for NewSheet {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let step = match &self.step {
            Step::Choose => None,
            Step::Template(step) => Some(step.clone().into_any_element()),
            Step::RunImage(step) => Some(step.clone().into_any_element()),
            Step::Paste(step) => Some(step.clone().into_any_element()),
        };
        if let Some(step) = step {
            return div().child(step);
        }
        let pick = |index: usize| {
            move |this: &mut Self, window: &mut Window, cx: &mut Context<Self>| {
                this.choose(NewOption::ALL[index], window, cx)
            }
        };
        let (one, two, three, four) = (pick(0), pick(1), pick(2), pick(3));
        sheet_view::render(self, cx)
            .track_focus(&self.focus)
            .key_context(CONTEXT)
            .on_action(cx.listener(|this, _: &SelectPrev, _, cx| this.step(-1, cx)))
            .on_action(cx.listener(|this, _: &SelectNext, _, cx| this.step(1, cx)))
            .on_action(cx.listener(|this, _: &Confirm, window, cx| {
                this.choose(NewOption::ALL[this.highlight], window, cx)
            }))
            .on_action(cx.listener(move |this, _: &Pick1, window, cx| one(this, window, cx)))
            .on_action(cx.listener(move |this, _: &Pick2, window, cx| two(this, window, cx)))
            .on_action(cx.listener(move |this, _: &Pick3, window, cx| three(this, window, cx)))
            .on_action(cx.listener(move |this, _: &Pick4, window, cx| four(this, window, cx)))
    }
}
