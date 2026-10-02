use std::cmp::Ordering;

use crate::extension::is_backend_project;

/// Volumes or networks that belong together in a list: one Compose project, or the
/// ones without a project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceGroup<T> {
    /// `None` for items that belong to no project.
    pub project: Option<String>,
    pub items: Vec<T>,
}

/// Groups `items` by project and keeps their order inside each group. Projects come
/// first, sorted by name. Items without a project come last.
pub fn group_by_project<'a, T: Clone + 'a>(
    items: impl IntoIterator<Item = &'a T>,
    project: impl Fn(&T) -> Option<&str>,
) -> Vec<ResourceGroup<T>> {
    let mut groups: Vec<ResourceGroup<T>> = Vec::new();
    for item in items {
        let key = project(item);
        match groups.iter_mut().find(|g| g.project.as_deref() == key) {
            Some(group) => group.items.push(item.clone()),
            None => groups.push(ResourceGroup {
                project: key.map(str::to_string),
                items: vec![item.clone()],
            }),
        }
    }
    groups.sort_by(|a, b| match (&a.project, &b.project) {
        (Some(a), Some(b)) => a.cmp(b),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    });
    groups
}

/// The groups a list shows: an extension's backend volumes and networks hide with
/// its containers unless `show_extensions` is on. Rendering and range selection
/// both use it, so a Shift-click cannot reach a hidden item. See feature 0025.
pub fn visible_groups<T>(
    mut groups: Vec<ResourceGroup<T>>,
    show_extensions: bool,
) -> Vec<ResourceGroup<T>> {
    if !show_extensions {
        groups.retain(|group| !group.project.as_deref().is_some_and(is_backend_project));
    }
    groups
}

#[cfg(test)]
mod tests;
