use crate::kubernetes::{ForwardKey, KubeService};
use crate::model::{Container, ContainerState};

use super::action::Destination;
use super::verb::Kinds;

/// The live names a command can use.
#[derive(Debug, Clone, Default)]
pub struct Catalog {
    pub containers: Vec<Container>,
    /// The Kubernetes services, read while k3s runs.
    pub kube_services: Vec<KubeService>,
    /// The project the window shows. Its services win over same-named services of
    /// other projects.
    pub current_project: Option<String>,
    /// True while k3s runs, so `forward` works.
    pub kubernetes: bool,
    /// True if `docker compose` is installed, so project commands work.
    pub compose: bool,
    /// The projects Captain knows that have no containers, for `up`.
    pub stopped_projects: Vec<String>,
}

/// What a name stands for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// A container by ID. A Compose service with one container is that container.
    Container(String),
    /// A Compose service with more than one container.
    Service {
        project: String,
        service: String,
    },
    Project(String),
    Port(u16),
    KubeService(ForwardKey),
    Page(Destination),
}

/// What a name matched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Resolution {
    Found(Target),
    Ambiguous(Vec<Target>),
    Missing,
}

impl Resolution {
    fn of(mut targets: Vec<Target>) -> Self {
        dedup(&mut targets);
        match targets.len() {
            0 => Resolution::Missing,
            1 => Resolution::Found(targets.remove(0)),
            _ => Resolution::Ambiguous(targets),
        }
    }
}

impl Catalog {
    /// The containers a command can name: Kubernetes pod containers and containers
    /// that are going away are left out.
    pub fn containers(&self) -> impl Iterator<Item = &Container> {
        self.containers
            .iter()
            .filter(|c| !c.is_kubernetes() && c.state != ContainerState::Removing)
    }

    pub fn container(&self, id: &str) -> Option<&Container> {
        self.containers().find(|c| c.id == id)
    }

    /// The containers of `project`, or of one of its services.
    pub fn members(&self, project: &str, service: Option<&str>) -> Vec<&Container> {
        self.containers()
            .filter(|c| c.compose_project.as_deref() == Some(project))
            .filter(|c| service.is_none() || c.compose.service.as_deref() == service)
            .collect()
    }

    /// The IDs of the containers behind `target`.
    pub fn ids(&self, target: &Target) -> Vec<String> {
        let members = match target {
            Target::Container(id) => return vec![id.clone()],
            Target::Service { project, service } => self.members(project, Some(service)),
            Target::Project(project) => self.members(project, None),
            _ => Vec::new(),
        };
        members.into_iter().map(|c| c.id.clone()).collect()
    }

    /// The Compose projects, in list order.
    pub fn projects(&self) -> Vec<&str> {
        let mut projects: Vec<&str> = Vec::new();
        for project in self
            .containers()
            .filter_map(|c| c.compose_project.as_deref())
        {
            if !projects.contains(&project) {
                projects.push(project);
            }
        }
        projects
    }

    /// The Compose services as `(project, service)`, in list order.
    fn services(&self) -> Vec<(&str, &str)> {
        let mut services: Vec<(&str, &str)> = Vec::new();
        for c in self.containers() {
            if let (Some(project), Some(service)) =
                (c.compose_project.as_deref(), c.compose.service.as_deref())
                && !services.contains(&(project, service))
            {
                services.push((project, service));
            }
        }
        services
    }

    fn service_target(&self, project: &str, service: &str) -> Option<Target> {
        match self.members(project, Some(service)).as_slice() {
            [] => None,
            [one] => Some(Target::Container(one.id.clone())),
            _ => Some(Target::Service {
                project: project.into(),
                service: service.into(),
            }),
        }
    }

    /// What `word` names among `kinds`. A service of the current project wins over
    /// services of other projects; any other clash is ambiguous.
    pub(super) fn resolve(&self, word: &str, kinds: Kinds) -> Resolution {
        if kinds.port
            && let Ok(port) = word.parse::<u16>()
        {
            return Resolution::of(vec![Target::Port(port)]);
        }
        if kinds.kube
            && let Some(rest) = word.strip_prefix("svc/")
        {
            return self.resolve_kube(rest);
        }
        let mut found = Vec::new();
        if kinds.page {
            found.extend(Destination::parse(word).map(Target::Page));
        }
        if kinds.service
            && let Some((project, service)) = word.split_once('/')
        {
            return Resolution::of(self.service_target(project, service).into_iter().collect());
        }
        if kinds.container {
            found.extend(
                self.containers()
                    .filter(|c| c.name == word)
                    .map(|c| Target::Container(c.id.clone())),
            );
        }
        if kinds.service {
            let services: Vec<(&str, &str)> = self
                .services()
                .into_iter()
                .filter(|s| s.1 == word)
                .collect();
            let current = self.current_project.as_deref();
            let chosen = match services.iter().find(|s| Some(s.0) == current) {
                Some(local) => vec![*local],
                None => services,
            };
            for (project, service) in chosen {
                found.extend(self.service_target(project, service));
            }
        }
        if kinds.project && self.projects().contains(&word) {
            found.push(Target::Project(word.into()));
        }
        if kinds.stopped && self.stopped_projects.iter().any(|name| name == word) {
            found.push(Target::Project(word.into()));
        }
        Resolution::of(found)
    }

    /// `name`, `namespace/name`, either with `:port`.
    fn resolve_kube(&self, rest: &str) -> Resolution {
        let (path, port) = match rest.rsplit_once(':') {
            Some((path, port)) => match port.parse::<u16>() {
                Ok(port) => (path, Some(port)),
                Err(_) => return Resolution::Missing,
            },
            None => (rest, None),
        };
        let (namespace, name) = match path.split_once('/') {
            Some((namespace, name)) => (Some(namespace), name),
            None => (None, path),
        };
        Resolution::of(
            self.kube_keys()
                .into_iter()
                .filter(|key| key.service == name)
                .filter(|key| namespace.is_none_or(|ns| key.namespace == ns))
                .filter(|key| port.is_none_or(|port| key.port == port))
                .map(Target::KubeService)
                .collect(),
        )
    }

    fn kube_keys(&self) -> Vec<ForwardKey> {
        self.kube_services
            .iter()
            .flat_map(|service| {
                service.ports.iter().map(|port| ForwardKey {
                    namespace: service.namespace.clone(),
                    service: service.name.clone(),
                    port: port.port,
                })
            })
            .collect()
    }

    /// Every name a word of `kinds` can take, with what it stands for. Each target
    /// gets its shortest name that is not ambiguous.
    pub(super) fn candidates(&self, kinds: Kinds) -> Vec<(String, Target)> {
        let mut out: Vec<(String, Target)> = Vec::new();
        let mut push = |text: String, target: Target| {
            if !out.iter().any(|(t, g)| *t == text && *g == target) {
                out.push((text, target));
            }
        };
        if kinds.service {
            for (project, service) in self.services() {
                let Some(target) = self.service_target(project, service) else {
                    continue;
                };
                let short = self.resolve(service, kinds) == Resolution::Found(target.clone());
                let text = if short {
                    service.to_string()
                } else {
                    format!("{project}/{service}")
                };
                push(text, target);
            }
        }
        if kinds.container {
            let mut containers: Vec<&Container> = self.containers().collect();
            containers.sort_by_key(|c| !c.state.is_active());
            for c in containers {
                push(c.name.clone(), Target::Container(c.id.clone()));
            }
        }
        if kinds.project {
            for project in self.projects() {
                push(project.to_string(), Target::Project(project.into()));
            }
        }
        if kinds.stopped {
            for project in &self.stopped_projects {
                push(project.clone(), Target::Project(project.clone()));
            }
        }
        if kinds.port && kinds.container {
            for port in self.containers().flat_map(|c| c.published_ports()) {
                push(port.to_string(), Target::Port(port));
            }
        }
        if kinds.kube && self.kubernetes {
            for key in self.kube_keys() {
                let target = Target::KubeService(key.clone());
                let (ns, name, port) = (&key.namespace, &key.service, key.port);
                let text = [
                    format!("svc/{name}"),
                    format!("svc/{ns}/{name}"),
                    format!("svc/{name}:{port}"),
                    format!("svc/{ns}/{name}:{port}"),
                ]
                .into_iter()
                .find(|text| self.resolve(text, kinds) == Resolution::Found(target.clone()));
                if let Some(text) = text {
                    push(text, target);
                }
            }
        }
        if kinds.page {
            for page in Destination::ALL {
                push(page.name().to_string(), Target::Page(page));
            }
        }
        out
    }
}

/// Removes repeated targets, keeping the first of each.
fn dedup(targets: &mut Vec<Target>) {
    let mut seen: Vec<Target> = Vec::new();
    targets.retain(|target| {
        let new = !seen.contains(target);
        if new {
            seen.push(target.clone());
        }
        new
    });
}

#[cfg(test)]
mod tests;
