use captain_core::model::ImageDetail;
use gpui_kit::base::actions::Cancel;
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

/// Opens the New sheet and its Open a folder card at once: the system's folder
/// dialog, then the sheet's check of the Compose files. Does nothing while a sheet
/// or dialog is open.
pub fn open_folder(
    workspace: Entity<Workspace>,
    project: Entity<ProjectView>,
    window: &mut Window,
    cx: &mut App,
) {
    if window.has_active_dialog(cx) {
        return;
    }
    let sheet = cx.new(|cx| NewSheet::new(SheetHost { workspace, project }, cx));
    show(sheet.clone(), window, cx);
    sheet.update(cx, |sheet, cx| {
        sheet.choose(NewOption::OpenFolder, window, cx)
    });
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
            // Return belongs to the sheet's cards and forms; it never closes it.
            .on_ok(|_, _, _| false)
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

    /// The step's view and its Back, or `None` on the cards.
    fn step_view(&self) -> Option<(AnyElement, StepBack)> {
        Some(match &self.step {
            Step::Choose => return None,
            Step::Template(step) => (
                step.clone().into_any_element(),
                back_of(step, TemplateStep::back),
            ),
            Step::RunImage(step) => (
                step.clone().into_any_element(),
                back_of(step, RunImage::back),
            ),
            Step::Paste(step) => (
                step.clone().into_any_element(),
                back_of(step, PasteStep::back),
            ),
        })
    }

    fn step(&mut self, by: isize, cx: &mut Context<Self>) {
        let count = NewOption::ALL.len() as isize;
        self.highlight = (self.highlight as isize + by).rem_euclid(count) as usize;
        cx.notify();
    }
}

/// Goes back one step from a step's form or list.
type StepBack = Box<dyn Fn(&mut Window, &mut App)>;

/// Runs `back` on `step` outside the sheet's update, since the first step's Back
/// updates the sheet.
fn back_of<T: 'static>(
    step: &Entity<T>,
    back: fn(&mut T, &mut Window, &mut Context<T>),
) -> StepBack {
    let step = step.clone();
    Box::new(move |window, cx| step.update(cx, |step, cx| back(step, window, cx)))
}

impl Render for NewSheet {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some((step, back)) = self.step_view() {
            // The sheet's focus stays inside the step: a click outside a field
            // focuses the sheet, so escape still goes back after the focused
            // field is gone, as when the compose.yaml preview replaces the form.
            return div()
                .track_focus(&self.focus)
                .key_context(CONTEXT)
                .on_action(move |_: &Cancel, window, cx| back(window, cx))
                .child(step);
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
