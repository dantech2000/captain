use gpui_kit::component::WindowExt;
use gpui_kit::*;

use super::keys::{CONTEXT, Confirm, Pick1, Pick2, Pick3, Pick4, SelectNext, SelectPrev};
use super::options::NewOption;
use super::sheet_view;
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

/// The New sheet: four ways to make a project. See
/// docs/features/0040-new-projects.md.
pub struct NewSheet {
    pub(super) workspace: Entity<Workspace>,
    pub(super) project: Entity<ProjectView>,
    /// The highlighted card, an index into [`NewOption::ALL`].
    pub(super) highlight: usize,
    pub(super) status: SheetStatus,
    pub(super) focus: FocusHandle,
    pub(super) check: Option<Task<()>>,
}

/// Opens the New sheet with Open a folder highlighted, the one option that works
/// in this build.
pub fn open(
    workspace: Entity<Workspace>,
    project: Entity<ProjectView>,
    window: &mut Window,
    cx: &mut App,
) {
    let sheet = cx.new(|cx| NewSheet {
        workspace,
        project,
        highlight: 2,
        status: SheetStatus::Idle,
        focus: cx.focus_handle(),
        check: None,
    });
    let focus = sheet.read(cx).focus.clone();
    window.open_dialog(cx, move |dialog, _, _| {
        dialog.w(px(560.)).margin_top(px(110.)).child(sheet.clone())
    });
    // The dialog takes focus as it opens; the sheet takes it back for its keys.
    window.defer(cx, move |window, cx| focus.focus(window, cx));
}

impl NewSheet {
    /// Opens the card `option`. Cards that are not ready do nothing.
    pub(super) fn choose(
        &mut self,
        option: NewOption,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(index) = NewOption::ALL.iter().position(|o| *o == option) {
            self.highlight = index;
        }
        if option == NewOption::OpenFolder && !matches!(self.status, SheetStatus::Checking(_)) {
            self.pick_folder(window, cx);
        }
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
