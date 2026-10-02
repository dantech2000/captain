use captain_core::model::ImageDetail;
use captain_core::new_project::{HubError, host_port_free, image_project_name, suggest_port};
use futures::StreamExt;
use gpui_kit::*;

use super::form_state::{Detail, RunImage};
use super::picker::Choice;
use crate::new_project::hub;
use crate::new_project::name_check::{published_ports, unique_name};

/// `reference` split into the repository, the tag, and the digest after `@`. A
/// registry port such as `localhost:5000/api` is not a tag; an ID has no tag.
pub fn split_reference(reference: &str) -> (String, String, Option<String>) {
    if reference.starts_with("sha256:") {
        return (reference.into(), String::new(), None);
    }
    let (name, digest) = match reference.split_once('@') {
        Some((name, digest)) => (name, Some(digest.to_string())),
        None => (reference, None),
    };
    match name.rsplit_once(':') {
        Some((repository, tag)) if !tag.contains('/') => (repository.into(), tag.into(), digest),
        _ => (name.into(), String::new(), digest),
    }
}

impl RunImage {
    /// Lists the engine's images for the picker and for "is it here?".
    pub(super) fn load_local(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(engine) = self.host.workspace.read(cx).engine() else {
            return;
        };
        let list = engine.list_images();
        self.side_task = Some(cx.spawn_in(window, async move |this, cx| {
            let Ok(images) = list.await else {
                return;
            };
            let local: Vec<String> = images
                .into_iter()
                .flat_map(|image| image.repo_tags)
                .filter(|tag| tag != "<none>:<none>")
                .collect();
            this.update(cx, |this, cx| {
                this.picker.update(cx, |list, cx| {
                    list.delegate_mut().set_local(local.clone());
                    cx.notify();
                });
                this.local = local;
                cx.notify();
            })
            .ok();
        }));
    }

    /// Shows the form for `reference`, which the Images page already inspected.
    pub fn choose_local(
        &mut self,
        reference: String,
        detail: ImageDetail,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.start_form(&reference, window, cx);
        self.prefill(&detail, window, cx);
    }

    /// Shows the form for the picked image.
    pub(super) fn choose(&mut self, choice: Choice, window: &mut Window, cx: &mut Context<Self>) {
        let reference = match &choice {
            Choice::Local(reference) | Choice::Typed(reference) => reference.clone(),
            Choice::Hub(repo) => format!("{}:latest", repo.name),
        };
        self.start_form(&reference, window, cx);
        if !matches!(choice, Choice::Local(_)) {
            self.load_tags(window, cx);
        }
        self.tag_changed(window, cx);
    }

    fn start_form(&mut self, reference: &str, window: &mut Window, cx: &mut Context<Self>) {
        let (repository, tag, digest) = split_reference(reference);
        let name = unique_name(&image_project_name(&repository), &self.host, cx);
        self.tags = self
            .local
            .iter()
            .filter_map(|local| {
                let (repo, tag, _) = split_reference(local);
                (repo == repository && !tag.is_empty()).then_some(tag)
            })
            .collect();
        self.repository = repository;
        self.digest = digest.map(|digest| (tag.clone(), digest));
        self.tag
            .update(cx, |input, cx| input.set_value(tag, window, cx));
        self.name
            .update(cx, |input, cx| input.set_value(name, window, cx));
        self.ports.clear();
        self.picking = false;
        self.detail = Detail::Missing;
        self.error = None;
        let focus = self.name.clone();
        window.defer(cx, move |window, cx| {
            focus.update(cx, |input, cx| input.focus(window, cx));
        });
        cx.notify();
    }

    /// Reads the image's ports when the engine has it.
    pub(super) fn tag_changed(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.picking {
            return;
        }
        let reference = self.reference(cx);
        let here = self.local.contains(&reference) || reference.starts_with("sha256:");
        if !here {
            self.inspect_task = None;
            self.detail = Detail::Missing;
            cx.notify();
            return;
        }
        let Some(engine) = self.host.workspace.read(cx).engine() else {
            return;
        };
        self.detail = Detail::Loading;
        let inspect = engine.inspect_image(&reference);
        self.inspect_task = Some(cx.spawn_in(window, async move |this, cx| {
            let result = inspect.await;
            this.update_in(cx, |this, window, cx| match result {
                Ok(detail) => this.prefill(&detail, window, cx),
                Err(error) => {
                    this.detail = Detail::Failed(error.to_string());
                    cx.notify();
                }
            })
            .ok();
        }));
        cx.notify();
    }

    /// One port row for each port the image exposes, on a free host port.
    fn prefill(&mut self, detail: &ImageDetail, window: &mut Window, cx: &mut Context<Self>) {
        let mut used = published_ports(&self.host, cx);
        self.ports.clear();
        for port in &detail.config.exposed_ports {
            let preferred = if port.port < 1024 {
                8000 + port.port
            } else {
                port.port
            };
            let host = suggest_port(preferred, &used, host_port_free);
            used.push(host);
            let container = match port.protocol.as_str() {
                "tcp" => port.port.to_string(),
                _ => port.to_string(),
            };
            self.add_port(host.to_string(), container, window, cx);
        }
        self.detail = Detail::Ready;
        cx.notify();
    }

    /// Pulls the image, then reads its ports.
    pub(super) fn pull(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy.is_some() {
            return;
        }
        let Some(engine) = self.host.workspace.read(cx).engine() else {
            return;
        };
        let reference = self.reference(cx);
        self.busy = Some(format!("Pulling {reference}\u{2026}"));
        self.error = None;
        let mut messages = engine.pull_image(&reference);
        self.task = Some(cx.spawn_in(window, async move |this, cx| {
            let mut failed = None;
            while let Some(message) = messages.next().await {
                if let Err(error) = message {
                    failed = Some(error.to_string());
                }
            }
            this.update_in(cx, |this, window, cx| {
                this.busy = None;
                match failed {
                    Some(error) => this.error = Some(pull_error(&reference, &error)),
                    None => {
                        this.local.push(reference);
                        this.tag_changed(window, cx);
                    }
                }
                cx.notify();
            })
            .ok();
        }));
        cx.notify();
    }

    /// Asks Docker Hub for the repository's newest tags.
    fn load_tags(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let repository = self.repository.clone();
        if let Some(tags) = hub::cached_tags(&repository, cx) {
            self.tags = tags;
            return;
        }
        let Ok(client) = hub::client(cx) else {
            return;
        };
        let tags = client.tags(&repository);
        self.side_task = Some(cx.spawn_in(window, async move |this, cx| {
            let result = tags.await;
            this.update(cx, |this, cx| {
                match result {
                    Ok(tags) => {
                        hub::keep_tags(repository, tags.clone(), cx);
                        this.tags = tags;
                    }
                    Err(HubError::RateLimited(wait)) => hub::wait(wait, cx),
                    Err(error) => tracing::info!(%error, "cannot list the image's tags"),
                }
                cx.notify();
            })
            .ok();
        }));
    }
}

/// A pull error, with the Docker Hub limit explained.
pub fn pull_error(reference: &str, error: &str) -> String {
    if error.contains("toomanyrequests") {
        return format!(
            "Docker Hub refused the pull of {reference}: too many pulls. Without a login it allows 100 pulls in 6 hours."
        );
    }
    format!("Cannot pull {reference}: {error}")
}

#[cfg(test)]
mod tests;
