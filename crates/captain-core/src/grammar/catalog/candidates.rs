use crate::grammar::action::Destination;
use crate::grammar::verb::Kinds;
use crate::model::Container;

use super::{Catalog, Resolution, Target};

impl Catalog {
    /// Every name a word of `kinds` can take, with what it stands for. Each target
    /// gets its shortest name that is not ambiguous.
    pub(in crate::grammar) fn candidates(&self, kinds: Kinds) -> Vec<(String, Target)> {
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
