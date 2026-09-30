use serde_json::Value;

/// What `up` would do to one container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeKind {
    /// The configuration changed: Compose replaces the container.
    Recreate,
    /// The service has no container yet.
    Create,
    /// The service left the files (with `--remove-orphans`).
    Remove,
    /// A stopped container starts again as it is.
    Start,
    /// It runs and stays as it is.
    Unchanged,
    /// An action Captain does not know, as Compose names it.
    Other(String),
}

impl ChangeKind {
    fn from_action(action: &str) -> Self {
        match action {
            "Running" => Self::Unchanged,
            "Recreate" | "Recreated" => Self::Recreate,
            "Creating" | "Created" => Self::Create,
            "Stopping" | "Stopped" | "Removing" | "Removed" => Self::Remove,
            "Starting" | "Started" => Self::Start,
            other => Self::Other(other.to_string()),
        }
    }

    /// The word the preview shows, for example `Recreate`.
    pub fn label(&self) -> &str {
        match self {
            Self::Recreate => "Recreate",
            Self::Create => "Create",
            Self::Remove => "Remove",
            Self::Start => "Start",
            Self::Unchanged => "No change",
            Self::Other(action) => action,
        }
    }
}

/// One container in the preview.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceChange {
    pub service: String,
    pub container: String,
    pub change: ChangeKind,
}

/// An image `up` would pull or build first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageStep {
    /// The image reference to pull, or the service to build.
    pub name: String,
    pub build: bool,
}

/// What `docker compose up --dry-run` says `up` would do.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UpPreview {
    /// Sorted by service, then container.
    pub changes: Vec<ServiceChange>,
    pub images: Vec<ImageStep>,
    /// Compose's warnings, for example about orphan containers.
    pub warnings: Vec<String>,
}

impl UpPreview {
    /// True if `up` would change nothing.
    pub fn is_empty(&self) -> bool {
        self.images.is_empty()
            && self
                .changes
                .iter()
                .all(|change| change.change == ChangeKind::Unchanged)
    }
}

/// Reads the stderr of `docker compose --progress json up -d --dry-run` for the
/// project `project`. `known` maps the names of existing containers to their
/// services, from their labels; a new container's service comes from its
/// `<project>-<service>-<n>` name.
///
/// Only the first action per container counts: after `Recreated`, the dry run
/// names fake containers `<12 hex>_<name>`, and some `Starting` events are
/// missing. See Compose issue #14269 for a dry run that hangs.
pub fn parse_dry_run(stderr: &str, project: &str, known: &[(String, String)]) -> UpPreview {
    let mut preview = UpPreview::default();
    let mut seen: Vec<String> = Vec::new();
    for line in stderr.lines() {
        let Ok(event) = serde_json::from_str::<Value>(line.trim()) else {
            continue;
        };
        let text = |key| event.get(key).and_then(Value::as_str).unwrap_or("");
        if text("level") == "warning" {
            preview.warnings.push(text("msg").to_string());
            continue;
        }
        let Some((kind, name)) = text("id").split_once(' ') else {
            continue;
        };
        if seen.iter().any(|id| id == text("id")) || is_fake_copy(name) {
            continue;
        }
        seen.push(text("id").to_string());
        match (kind, text("text")) {
            ("Container", action) => preview.changes.push(ServiceChange {
                service: service_of(name, project, known),
                container: name.to_string(),
                change: ChangeKind::from_action(action),
            }),
            ("Image", "Pulling") => preview.images.push(ImageStep {
                name: name.to_string(),
                build: false,
            }),
            ("Image", "Building") => preview.images.push(ImageStep {
                name: name
                    .strip_prefix(project)
                    .and_then(|rest| rest.strip_prefix('-'))
                    .unwrap_or(name)
                    .to_string(),
                build: true,
            }),
            _ => {}
        }
    }
    preview
        .changes
        .sort_by(|a, b| (&a.service, &a.container).cmp(&(&b.service, &b.container)));
    preview
}

/// `00fccd82ddf9_shop-web-1`, a name the dry run makes up after `Recreated`.
fn is_fake_copy(name: &str) -> bool {
    name.split_once('_').is_some_and(|(prefix, _)| {
        prefix.len() == 12 && prefix.bytes().all(|b| b.is_ascii_hexdigit())
    })
}

fn service_of(container: &str, project: &str, known: &[(String, String)]) -> String {
    if let Some((_, service)) = known.iter().find(|(name, _)| name == container) {
        return service.clone();
    }
    container
        .strip_prefix(project)
        .and_then(|rest| rest.strip_prefix('-'))
        .and_then(|rest| rest.rsplit_once('-'))
        .filter(|(_, n)| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
        .map_or_else(|| container.to_string(), |(service, _)| service.to_string())
}

#[cfg(test)]
mod tests;
