/// A file name for `name` that `exists` says is free: `name`, else `name (1)`,
/// `name (2)`, and so on, with the number before the extension.
pub fn save_name(name: &str, exists: impl Fn(&str) -> bool) -> String {
    if !exists(name) {
        return name.to_string();
    }
    // A leading dot starts a hidden name, not an extension.
    let (stem, extension) = match name.rfind('.') {
        Some(ix) if ix > 0 => name.split_at(ix),
        _ => (name, ""),
    };
    (1..)
        .map(|n| format!("{stem} ({n}){extension}"))
        .find(|candidate| !exists(candidate))
        .expect("an unused name")
}

#[cfg(test)]
mod tests;
