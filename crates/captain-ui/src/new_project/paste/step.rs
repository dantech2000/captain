use captain_core::new_project::{ComposeDoc, NewFile, RunConversion, convert_docker_run};
use gpui_kit::component::input::{InputEvent, InputState, TextareaState};
use gpui_kit::*;

use crate::new_project::finish::create_project;
use crate::new_project::form::{new_input, value};
use crate::new_project::host::SheetHost;
use crate::new_project::name_check::{name_problem, unique_name};
use crate::new_project::new_sheet::NewSheet;

/// The pasted command, its conversion, and the project name.
pub struct PasteStep {
    pub(super) host: SheetHost,
    pub(super) sheet: WeakEntity<NewSheet>,
    pub(super) command: Entity<TextareaState>,
    pub(super) name: Entity<InputState>,
    /// `None` while the field is empty.
    pub(super) conversion: Option<Result<RunConversion, String>>,
    /// True once the user typed a name; a new command then keeps it.
    name_typed: bool,
    pub(super) show_compose: bool,
    pub(super) error: Option<String>,
    _subscriptions: Vec<Subscription>,
}

impl PasteStep {
    pub fn new(
        host: SheetHost,
        sheet: WeakEntity<NewSheet>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let command = cx.new(|cx| {
            // Six lines, growing to twelve, then it scrolls.
            TextareaState::new(window, cx)
                .auto_grow(6, 12)
                .placeholder("docker run -d -p 8080:80 nginx")
        });
        let name = new_input("Project name", "", window, cx);
        let subscriptions = vec![
            cx.subscribe_in(&command, window, |this, _, event, window, cx| {
                if matches!(event, InputEvent::Change) {
                    this.convert(window, cx);
                }
            }),
            cx.subscribe(&name, |this, _, event, _| {
                if matches!(event, InputEvent::Change) {
                    this.name_typed = true;
                }
            }),
        ];
        let focus = command.clone();
        window.defer(cx, move |window, cx| {
            focus.update(cx, |input, cx| input.focus(window, cx));
        });
        Self {
            host,
            sheet,
            command,
            name,
            conversion: None,
            name_typed: false,
            show_compose: false,
            error: None,
            _subscriptions: subscriptions,
        }
    }

    /// Converts the command again, and names the project after it until the user
    /// types a name.
    fn convert(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let text = self.command.read(cx).value().to_string();
        self.conversion = (!text.trim().is_empty()).then(|| convert_docker_run(&text));
        if !self.name_typed
            && let Some(Ok(conversion)) = &self.conversion
        {
            let base = conversion
                .container_name
                .clone()
                .unwrap_or_else(|| conversion.service_name.clone());
            let name = unique_name(&base, &self.host, cx);
            self.name
                .update(cx, |input, cx| input.set_value(name, window, cx));
        }
        self.error = None;
        cx.notify();
    }

    /// The Compose file and the project name, or the first problem.
    pub(super) fn compose(&self, cx: &App) -> Result<(String, String), String> {
        let conversion = match &self.conversion {
            None => return Err("Paste a docker run command.".into()),
            Some(Err(error)) => return Err(error.clone()),
            Some(Ok(conversion)) => conversion,
        };
        let name = value(&self.name, cx);
        if let Some(problem) = name_problem(&name, &self.host, cx) {
            return Err(problem);
        }
        let doc = ComposeDoc {
            name: Some(name.clone()),
            services: vec![(conversion.service_name.clone(), conversion.service.clone())],
        };
        Ok((name, doc.to_yaml()))
    }

    pub(crate) fn back(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(sheet) = self.sheet.upgrade() {
            sheet.update(cx, |sheet, cx| sheet.back(window, cx));
        }
    }

    pub(super) fn create(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let result = self.compose(cx).and_then(|(name, text)| {
            let files = [NewFile::new("compose.yaml", text)];
            create_project(&self.host, &name, &files, window, cx)
        });
        self.error = result.err();
        cx.notify();
    }
}
