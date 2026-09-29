//! Absolute paths inside a container. They always use `/`, whatever the host OS.

/// One part of the path bar: its label and the folder it opens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Crumb {
    pub label: String,
    pub path: String,
}

/// `name` inside the folder `dir`.
pub fn join_path(dir: &str, name: &str) -> String {
    format!("{}/{name}", dir.trim_end_matches('/'))
}

/// The folder that holds `path`. The root has none.
pub fn parent_path(path: &str) -> Option<String> {
    let trimmed = path.trim_end_matches('/');
    if trimmed.is_empty() {
        return None;
    }
    match trimmed.rfind('/') {
        Some(0) | None => Some("/".into()),
        Some(ix) => Some(trimmed[..ix].into()),
    }
}

/// The root, then each folder on the way to `path`.
pub fn breadcrumbs(path: &str) -> Vec<Crumb> {
    let mut crumbs = vec![Crumb {
        label: "/".into(),
        path: "/".into(),
    }];
    let mut current = String::new();
    for part in path.split('/').filter(|part| !part.is_empty()) {
        current.push('/');
        current.push_str(part);
        crumbs.push(Crumb {
            label: part.into(),
            path: current.clone(),
        });
    }
    crumbs
}

#[cfg(test)]
mod tests;
