use std::cmp::Ordering;

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

#[cfg(test)]
mod tests;
