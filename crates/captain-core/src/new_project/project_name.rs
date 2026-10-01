/// Why a project name is not valid, or `None` when it is. Compose project names
/// have only lowercase letters, digits, `-`, and `_`, and start with a letter or a
/// digit (<https://docs.docker.com/compose/how-tos/project-name/>). The name is
/// also the project's folder.
pub fn project_name_error(name: &str) -> Option<&'static str> {
    let Some(first) = name.chars().next() else {
        return Some("Enter a project name.");
    };
    if !(first.is_ascii_lowercase() || first.is_ascii_digit()) {
        return Some("Start the name with a lowercase letter or a digit.");
    }
    let valid = name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_');
    if !valid {
        return Some("Use only lowercase letters, digits, - and _.");
    }
    None
}

/// `text` as a project name: lowercased, with each run of other characters as
/// one `-`, and nothing but letters and digits at the start. `project` when
/// nothing is left. For example the image `bitnami/redis:7` gives `redis`.
pub fn to_project_name(text: &str) -> String {
    let mut name = String::new();
    for c in text.chars().flat_map(char::to_lowercase) {
        if c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' {
            name.push(c);
        } else if !name.is_empty() && !name.ends_with('-') {
            name.push('-');
        }
    }
    let name = name.trim_start_matches('_').trim_end_matches(['-', '_']);
    if name.is_empty() {
        "project".into()
    } else {
        name.to_string()
    }
}

/// The repository part of an image reference, as a project name: no registry, no
/// namespace, no tag or digest. `ghcr.io/acme/api:1.2` gives `api`.
pub fn image_project_name(image: &str) -> String {
    let path = image.split('@').next().unwrap_or(image);
    let last = path.rsplit('/').next().unwrap_or(path);
    let repo = last.split(':').next().unwrap_or(last);
    to_project_name(repo)
}

/// `base`, or `base-2`, `base-3`, and so on: the first that `taken` refuses.
pub fn unique_project_name(base: &str, taken: impl Fn(&str) -> bool) -> String {
    if !taken(base) {
        return base.to_string();
    }
    (2..)
        .map(|n| format!("{base}-{n}"))
        .find(|name| !taken(name))
        .expect("some number is free")
}

#[cfg(test)]
mod tests;
