use captain_core::new_project::{
    Template, TemplateValues, host_port_free, password_error, random_password, suggest_port,
};
use gpui_kit::component::IndexPath;
use gpui_kit::component::input::InputState;
use gpui_kit::component::list::{ListEvent, ListState};
use gpui_kit::*;

use super::picker::TemplatePicker;
use crate::new_project::finish::create_project;
use crate::new_project::form::{new_input, value};
use crate::new_project::host::SheetHost;
use crate::new_project::name_check::{name_problem, published_ports, unique_name};
use crate::new_project::new_sheet::NewSheet;

/// Start from a template: the picker, then the form of the chosen template.
pub struct TemplateStep {
    pub(super) host: SheetHost,
    pub(super) sheet: WeakEntity<NewSheet>,
    pub(super) list: Entity<ListState<TemplatePicker>>,
    pub(super) form: Option<TemplateForm>,
    pub(super) show_compose: bool,
    pub(super) error: Option<String>,
    _picked: Subscription,
}

/// The fields of the chosen template.
pub struct TemplateForm {
    pub template: &'static Template,
    pub name: Entity<InputState>,
    /// One host port for each of the template's ports.
    pub ports: Vec<Entity<InputState>>,
    pub user: Option<Entity<InputState>>,
    pub password: Option<Entity<InputState>>,
}

impl TemplateStep {
    pub fn new(
        host: SheetHost,
        sheet: WeakEntity<NewSheet>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let list = cx.new(|cx| {
            let mut list = ListState::new(TemplatePicker::new(), window, cx).searchable(true);
            list.set_selected_index(Some(IndexPath::default()), window, cx);
            list
        });
        let picked = cx.subscribe_in(&list, window, |this, list, event, window, cx| {
            if let ListEvent::Confirm(ix) = event
                && let Some(template) = list.read(cx).delegate().template(*ix)
            {
                this.pick(template, window, cx);
            }
        });
        let focus = list.clone();
        window.defer(cx, move |window, cx| {
            focus.update(cx, |list, cx| list.focus(window, cx));
        });
        Self {
            host,
            sheet,
            list,
            form: None,
            show_compose: false,
            error: None,
            _picked: picked,
        }
    }

    /// Shows the form of `template`, with a free name, free ports, and a new
    /// password.
    fn pick(&mut self, template: &'static Template, window: &mut Window, cx: &mut Context<Self>) {
        let name = unique_name(template.service, &self.host, cx);
        let mut used = published_ports(&self.host, cx);
        let ports = (0..template.ports.len())
            .map(|index| {
                let port = suggest_port(template.preferred_port(index), &used, host_port_free);
                used.push(port);
                new_input("Host port", port.to_string(), window, cx)
            })
            .collect();
        let user = template
            .user
            .map(|(_, default)| new_input("User name", default, window, cx));
        let password = template.password.map(|_| {
            let password = random_password(24);
            cx.new(|cx| {
                InputState::new(window, cx)
                    .masked(true)
                    .default_value(password)
            })
        });
        let name = new_input("Project name", name, window, cx);
        let focus = name.clone();
        window.defer(cx, move |window, cx| {
            focus.update(cx, |input, cx| input.focus(window, cx));
        });
        self.form = Some(TemplateForm {
            template,
            name,
            ports,
            user,
            password,
        });
        self.error = None;
        cx.notify();
    }

    /// From the form back to the list; from the list back to the four cards.
    pub(super) fn back(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.form.take().is_some() {
            self.show_compose = false;
            self.error = None;
            self.list.update(cx, |list, cx| list.focus(window, cx));
            cx.notify();
            return;
        }
        if let Some(sheet) = self.sheet.upgrade() {
            sheet.update(cx, |sheet, cx| sheet.back(window, cx));
        }
    }

    /// The form's values, or the first problem with them.
    pub(super) fn values(&self, cx: &App) -> Result<(&'static Template, TemplateValues), String> {
        let form = self.form.as_ref().ok_or("Pick a template.")?;
        let name = value(&form.name, cx);
        if let Some(problem) = name_problem(&name, &self.host, cx) {
            return Err(problem);
        }
        let mut ports: Vec<u16> = Vec::new();
        for (input, port) in form.ports.iter().zip(form.template.ports) {
            let host = value(input, cx)
                .parse::<u16>()
                .ok()
                .filter(|p| *p > 0)
                .ok_or_else(|| format!("{}: enter a number from 1 to 65535.", port.label))?;
            if ports.contains(&host) {
                return Err(format!("Host port {host} is used twice."));
            }
            ports.push(host);
        }
        let user = form.user.as_ref().map(|i| value(i, cx)).unwrap_or_default();
        let plain = |c: char| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.');
        if form.user.is_some() && (user.is_empty() || !user.chars().all(plain)) {
            return Err("User name: use letters, digits, and - _ . only.".into());
        }
        let password = form
            .password
            .as_ref()
            .map(|i| value(i, cx))
            .unwrap_or_default();
        if form.password.is_some()
            && let Some(problem) = password_error(&password)
        {
            return Err(format!("Password: {problem}"));
        }
        Ok((
            form.template,
            TemplateValues {
                name,
                ports,
                user,
                password,
            },
        ))
    }

    /// Writes the project and opens its files.
    pub(super) fn create(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let result = self.values(cx).and_then(|(template, values)| {
            create_project(
                &self.host,
                &values.name,
                &template.files(&values),
                window,
                cx,
            )
        });
        self.error = result.err();
        cx.notify();
    }
}
