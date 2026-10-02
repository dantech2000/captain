use captain_core::model::RestartPolicy;
use captain_core::store::{PortRow, RunForm, VolumeRow};
use gpui_kit::component::IndexPath;
use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::component::list::{ListEvent, ListState};
use gpui_kit::*;

use super::picker::ImagePicker;
use crate::images::ImagesState;
use crate::new_project::form::{new_input, value};
use crate::new_project::host::SheetHost;
use crate::new_project::new_sheet::NewSheet;

/// What Captain knows of the chosen image's settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Detail {
    /// This engine does not have the image; a pull reads its ports.
    Missing,
    Loading,
    Ready,
    Failed(String),
}

/// Run an image: the image picker, then the form. With Save as a project, the
/// form writes a one-service Compose project; without it, it runs a plain
/// container, as `docker run -d` does.
pub struct RunImage {
    pub(super) host: SheetHost,
    pub(super) sheet: WeakEntity<NewSheet>,
    /// The Images page, when it opened the sheet. It shows the Started notice.
    pub(super) images: Option<Entity<ImagesState>>,
    pub(super) picker: Entity<ListState<ImagePicker>>,
    /// True while the picker shows, before an image is chosen.
    pub(super) picking: bool,
    /// The engine's image references, such as `nginx:1.27`.
    pub(super) local: Vec<String>,
    /// The chosen repository without its tag, such as `postgres`, or an ID.
    pub(super) repository: String,
    pub(super) tag: Entity<InputState>,
    /// The digest of a chosen `repo@sha256:...` reference, with the tag it came
    /// with. It applies while the tag field still holds that tag.
    pub(super) digest: Option<(String, String)>,
    /// Tags to pick from, from Docker Hub or the engine.
    pub(super) tags: Vec<String>,
    pub(super) detail: Detail,
    pub(super) name: Entity<InputState>,
    /// Host and container port fields.
    pub(super) ports: Vec<(Entity<InputState>, Entity<InputState>)>,
    /// One `KEY=value` field for each variable.
    pub(super) env: Vec<Entity<InputState>>,
    /// Volume name or folder, and container path fields.
    pub(super) volumes: Vec<(Entity<InputState>, Entity<InputState>)>,
    pub(super) restart: RestartPolicy,
    pub(super) auto_remove: bool,
    pub(super) save_as_project: bool,
    pub(super) show_compose: bool,
    /// The last validation or engine error.
    pub(super) error: Option<String>,
    /// What runs now, such as a pull. The buttons wait for it.
    pub(super) busy: Option<String>,
    /// A pull or a run. Only its own end clears `busy`, so nothing else may
    /// replace it.
    pub(super) task: Option<Task<()>>,
    /// Reading the image's ports. A new tag replaces it.
    pub(super) inspect_task: Option<Task<()>>,
    pub(super) side_task: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl RunImage {
    pub fn new(
        host: SheetHost,
        sheet: WeakEntity<NewSheet>,
        images: Option<Entity<ImagesState>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let picker = cx.new(|cx| {
            let mut list = ListState::new(ImagePicker::new(), window, cx).searchable(true);
            list.set_selected_index(Some(IndexPath::default()), window, cx);
            list
        });
        let tag = new_input("latest", "", window, cx);
        let subscriptions = vec![
            cx.subscribe_in(&picker, window, |this, picker, event, window, cx| {
                if let ListEvent::Confirm(ix) = event
                    && let Some(choice) = picker.read(cx).delegate().choice(*ix)
                {
                    this.choose(choice, window, cx);
                }
            }),
            cx.subscribe_in(&tag, window, |this, _, event, window, cx| {
                if matches!(event, InputEvent::Change) {
                    this.tag_changed(window, cx);
                }
            }),
        ];
        let focus = picker.clone();
        window.defer(cx, move |window, cx| {
            focus.update(cx, |list, cx| list.focus(window, cx));
        });
        let mut this = Self {
            host,
            sheet,
            images,
            picker,
            picking: true,
            local: Vec::new(),
            repository: String::new(),
            tag,
            digest: None,
            tags: Vec::new(),
            detail: Detail::Missing,
            name: new_input("Project name", "", window, cx),
            ports: Vec::new(),
            env: Vec::new(),
            volumes: Vec::new(),
            restart: RestartPolicy::No,
            auto_remove: false,
            save_as_project: true,
            show_compose: false,
            error: None,
            busy: None,
            task: None,
            inspect_task: None,
            side_task: None,
            _subscriptions: subscriptions,
        };
        this.load_local(window, cx);
        this
    }

    /// The image reference the form runs: `repository:tag`, with `@digest` when
    /// the chosen image had one and the tag did not change, or the ID.
    pub(super) fn reference(&self, cx: &App) -> String {
        let tag = value(&self.tag, cx);
        let mut reference = match tag.as_str() {
            "" => self.repository.clone(),
            tag => format!("{}:{tag}", self.repository),
        };
        if let Some(digest) = self.pinned_digest(cx) {
            reference = format!("{reference}@{digest}");
        }
        reference
    }

    /// The digest the form keeps, while the tag field holds the tag it came with.
    pub(super) fn pinned_digest(&self, cx: &App) -> Option<&str> {
        let (tag, digest) = self.digest.as_ref()?;
        (value(&self.tag, cx) == *tag).then_some(digest.as_str())
    }

    /// The form as the fields hold it now.
    pub(super) fn form(&self, cx: &App) -> RunForm {
        let text = |input: &Entity<InputState>| value(input, cx);
        RunForm {
            image: self.reference(cx),
            name: text(&self.name),
            ports: self
                .ports
                .iter()
                .map(|(host, container)| PortRow {
                    host: text(host),
                    container: text(container),
                })
                .collect(),
            env: self.env.iter().map(text).collect(),
            volumes: self
                .volumes
                .iter()
                .map(|(source, target)| VolumeRow {
                    source: text(source),
                    target: text(target),
                })
                .collect(),
            auto_remove: self.auto_remove && !self.save_as_project,
            restart: self.restart,
        }
    }

    pub(super) fn add_port(
        &mut self,
        host: String,
        container: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let host = new_input("8080", host, window, cx);
        let container = new_input("80/tcp", container, window, cx);
        self.ports.push((host, container));
        cx.notify();
    }

    pub(super) fn add_env(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.env.push(new_input("KEY=value", "", window, cx));
        cx.notify();
    }

    pub(super) fn add_volume(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let source = new_input("data or ./folder", "", window, cx);
        let target = new_input("/path/in/container", "", window, cx);
        self.volumes.push((source, target));
        cx.notify();
    }

    /// Removes row `ix` of the ports (0), environment (1), or volumes (2).
    pub(super) fn remove_row(&mut self, list: u8, ix: usize, cx: &mut Context<Self>) {
        match list {
            0 if ix < self.ports.len() => drop(self.ports.remove(ix)),
            1 if ix < self.env.len() => drop(self.env.remove(ix)),
            2 if ix < self.volumes.len() => drop(self.volumes.remove(ix)),
            _ => return,
        }
        cx.notify();
    }

    pub(super) fn set_restart(&mut self, restart: RestartPolicy, cx: &mut Context<Self>) {
        self.restart = restart;
        cx.notify();
    }

    pub(super) fn set_auto_remove(&mut self, auto_remove: bool, cx: &mut Context<Self>) {
        self.auto_remove = auto_remove;
        cx.notify();
    }

    pub(super) fn set_save_as_project(&mut self, on: bool, cx: &mut Context<Self>) {
        self.save_as_project = on;
        self.show_compose &= on;
        self.error = None;
        cx.notify();
    }

    /// From the form back to the picker; from the picker back to the four cards.
    pub(crate) fn back(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.picking {
            self.picking = true;
            self.error = None;
            self.picker.update(cx, |list, cx| list.focus(window, cx));
            cx.notify();
            return;
        }
        if let Some(sheet) = self.sheet.upgrade() {
            sheet.update(cx, |sheet, cx| sheet.back(window, cx));
        }
    }
}
