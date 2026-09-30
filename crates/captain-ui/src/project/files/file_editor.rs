use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use captain_core::ProjectRunner;
use captain_core::model::ComposeProject;
use captain_core::project_files::{
    EditableFile, FileKind, LineProblem, SaveError, TextVersion, disk_version, read_text, save_text,
};
use gpui_kit::component::input::{EditorState, InputEvent};
use gpui_kit::*;

use super::editor_panel;
use super::grammar::{DOCKERFILE, register_dockerfile};
use super::providers::ComposeHelp;
use crate::project::ProjectView;

/// How often an open file is compared with the disk. A read of a small file is
/// cheap, and a poll works the same on every OS and through a symlink.
const WATCH_EVERY: Duration = Duration::from_secs(2);

/// Whether the file on disk still is the text the editor loaded or saved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiskState {
    Same,
    /// Another program changed the file while it had unsaved edits here.
    Changed,
    /// The file cannot be read any more, for example after it was deleted.
    Missing,
}

/// One open file of a project: its editor, the version it is based on, and the
/// results of its checks.
pub struct FileEditor {
    pub file: EditableFile,
    pub(super) project: ComposeProject,
    pub(super) runner: Option<Arc<dyn ProjectRunner>>,
    pub editor: Entity<EditorState>,
    /// The disk version the text is based on; `None` if the file could not be read.
    loaded: Option<TextVersion>,
    saved_text: SharedString,
    pub dirty: bool,
    pub disk: DiskState,
    /// Why the last read or save failed.
    pub error: Option<String>,
    pub(super) problems: Vec<LineProblem>,
    /// Why the last check could not run, for example a Buildx without checks.
    pub(super) check_error: Option<String>,
    pub(super) checking: bool,
    /// What the Project page is doing with this file, for example "Checking what
    /// changes...". The buttons wait meanwhile.
    pub busy: Option<SharedString>,
    /// Why the last Save and apply or Rebuild failed. The next action clears it.
    pub action_error: Option<SharedString>,
    pub(super) check: Option<Task<()>>,
    /// The file changed on disk and has no unsaved edits: reload on the next frame,
    /// which has a window.
    reload_pending: bool,
    /// The Project page, for Save and apply.
    pub(super) view: WeakEntity<ProjectView>,
    _watch: Task<()>,
    _changes: Subscription,
}

impl FileEditor {
    pub fn new(
        file: EditableFile,
        project: ComposeProject,
        runner: Option<Arc<dyn ProjectRunner>>,
        view: WeakEntity<ProjectView>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        register_dockerfile();
        let language = match file.kind {
            FileKind::Compose => "yaml",
            FileKind::Dockerfile => DOCKERFILE,
        };
        let editor = cx.new(|cx| {
            EditorState::new(window, cx)
                .language(language)
                .line_number(true)
                .searchable(true)
        });
        if file.kind == FileKind::Compose {
            editor.update(cx, |state, _| {
                let help = Rc::new(ComposeHelp);
                state.lsp_mut().completion_provider = Some(help.clone());
                state.lsp_mut().hover_provider = Some(help);
            });
        }
        let changes = cx.subscribe_in(&editor, window, |this, _, event: &InputEvent, _, cx| {
            if matches!(event, InputEvent::Change) {
                this.edited(cx);
            }
        });
        let path = file.path.clone();
        let watch = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(WATCH_EVERY).await;
                let path = path.clone();
                let version = cx
                    .background_executor()
                    .spawn(async move { disk_version(&path) })
                    .await;
                if this
                    .update(cx, |this, cx| this.compare(version, cx))
                    .is_err()
                {
                    break;
                }
            }
        });
        let mut this = Self {
            file,
            project,
            runner,
            editor,
            loaded: None,
            saved_text: SharedString::default(),
            dirty: false,
            disk: DiskState::Same,
            error: None,
            problems: Vec::new(),
            check_error: None,
            checking: false,
            busy: None,
            action_error: None,
            check: None,
            reload_pending: false,
            view,
            _watch: watch,
            _changes: changes,
        };
        this.load(window, cx);
        this
    }

    /// Takes the newest facts about the file and its project, for example a service
    /// renamed in the Compose file or a new build context. The text and its undo
    /// history stay; the checks run again when something changed.
    pub fn rebind(&mut self, file: EditableFile, project: ComposeProject, cx: &mut Context<Self>) {
        if self.file == file && self.project == project {
            return;
        }
        self.file = file;
        self.project = project;
        self.schedule_check(cx);
        cx.notify();
    }

    /// The file's name in the project folder, for example `app/Dockerfile`.
    pub fn label(&self) -> String {
        let dir = self.project.working_dir.as_deref().unwrap_or_default();
        self.file.label(std::path::Path::new(dir))
    }

    /// Reads the file from disk, dropping unsaved edits.
    pub fn load(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let (text, loaded, error) = match read_text(&self.file.path) {
            Ok(read) => (read.text, Some(read.version), None),
            Err(error) => (
                String::new(),
                None,
                Some(format!("Cannot read the file: {error}")),
            ),
        };
        let text = SharedString::from(text);
        self.editor
            .update(cx, |state, cx| state.set_value(text.clone(), window, cx));
        self.saved_text = text;
        self.disk = if loaded.is_some() {
            DiskState::Same
        } else {
            DiskState::Missing
        };
        self.loaded = loaded;
        self.error = error;
        self.dirty = false;
        self.schedule_check(cx);
        cx.notify();
    }

    /// Writes the text if the file on disk is still the version it is based on.
    /// Returns true when the file is saved.
    pub fn save(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(loaded) = self.loaded.clone() else {
            return false;
        };
        let text = self.editor.read(cx).value();
        let saved = match save_text(&self.file.path, &text, &loaded) {
            Ok(version) => {
                self.loaded = Some(version);
                self.saved_text = text;
                self.dirty = false;
                self.error = None;
                true
            }
            Err(SaveError::Changed) => {
                self.disk = DiskState::Changed;
                self.error = Some(
                    "Not saved: the file changed on disk. Reload it, or keep your text and \
                     save again."
                        .into(),
                );
                false
            }
            Err(SaveError::Io(error)) => {
                self.error = Some(format!("Not saved: {error}"));
                false
            }
        };
        cx.notify();
        saved
    }

    /// Keeps the unsaved text after an outside change: the next Save replaces the
    /// file on disk now.
    pub fn keep_mine(&mut self, cx: &mut Context<Self>) {
        if let Some(version) = disk_version(&self.file.path) {
            self.loaded = Some(version);
            self.disk = DiskState::Same;
            self.error = None;
            cx.notify();
        }
    }

    fn edited(&mut self, cx: &mut Context<Self>) {
        self.dirty = self.editor.read(cx).value() != self.saved_text;
        self.schedule_check(cx);
        cx.notify();
    }

    /// Follows the disk: a file without unsaved edits takes the new text on the
    /// next frame; one with edits shows the reload bar instead, so nothing is lost
    /// either way.
    fn compare(&mut self, version: Option<TextVersion>, cx: &mut Context<Self>) {
        let disk = match &version {
            None => DiskState::Missing,
            Some(version) if Some(version) == self.loaded.as_ref() => DiskState::Same,
            Some(_) => DiskState::Changed,
        };
        if disk == DiskState::Changed && !self.dirty {
            self.reload_pending = true;
            cx.notify();
        } else if disk != self.disk {
            self.disk = disk;
            cx.notify();
        }
    }
}

impl Render for FileEditor {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if std::mem::take(&mut self.reload_pending) {
            // An edit may have come in since the disk changed; then ask instead.
            if self.dirty {
                self.disk = DiskState::Changed;
            } else {
                self.load(window, cx);
            }
        }
        editor_panel::render(self, cx)
    }
}
