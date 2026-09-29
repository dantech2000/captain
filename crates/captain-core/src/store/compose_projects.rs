use crate::model::{ComposeProject, ComposeService, Container};

/// Groups containers into Compose projects by their labels. Projects are sorted by
/// name, services by name, and containers by name. Containers without a project are
/// left out. A container without a service label is its own service, named after
/// the container.
pub fn compose_projects(containers: &[Container]) -> Vec<ComposeProject> {
    let mut projects: Vec<ComposeProject> = Vec::new();
    for container in containers {
        let Some(name) = &container.compose_project else {
            continue;
        };
        let index = match projects.iter().position(|p| &p.name == name) {
            Some(index) => index,
            None => {
                projects.push(ComposeProject {
                    name: name.clone(),
                    working_dir: None,
                    config_files: Vec::new(),
                    services: Vec::new(),
                });
                projects.len() - 1
            }
        };
        add(&mut projects[index], container);
    }
    projects.sort_by(|a, b| a.name.cmp(&b.name));
    for project in &mut projects {
        project.services.sort_by(|a, b| a.name.cmp(&b.name));
        for service in &mut project.services {
            service.containers.sort_by(|a, b| a.name.cmp(&b.name));
        }
    }
    projects
}

/// The project called `name`, if any container belongs to it.
pub fn compose_project(containers: &[Container], name: &str) -> Option<ComposeProject> {
    let members: Vec<Container> = containers
        .iter()
        .filter(|c| c.compose_project.as_deref() == Some(name))
        .cloned()
        .collect();
    compose_projects(&members).into_iter().next()
}

/// Adds `container` to its service. The first container with a working directory or
/// config files sets them for the project.
fn add(project: &mut ComposeProject, container: &Container) {
    let labels = &container.compose;
    if project.working_dir.is_none() {
        project.working_dir = labels.working_dir.clone().filter(|d| !d.is_empty());
    }
    if project.config_files.is_empty() {
        project.config_files = labels.config_files.clone();
    }
    let service = labels
        .service
        .clone()
        .unwrap_or_else(|| container.name.clone());
    match project.services.iter_mut().find(|s| s.name == service) {
        Some(existing) => existing.containers.push(container.clone()),
        None => project.services.push(ComposeService {
            name: service,
            containers: vec![container.clone()],
        }),
    }
}

#[cfg(test)]
mod tests;
