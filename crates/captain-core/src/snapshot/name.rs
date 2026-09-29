/// The longest name, in characters. Rancher Desktop uses the same limit.
pub const MAX_NAME_CHARS: usize = 250;

/// Checks a new snapshot name against the rules and the `existing` names.
pub fn check_name<'a>(
    name: &str,
    mut existing: impl Iterator<Item = &'a str>,
) -> Result<(), String> {
    if name.is_empty() {
        return Err("Enter a name.".into());
    }
    if name.trim() != name {
        return Err("The name cannot start or end with a space.".into());
    }
    if name.chars().count() > MAX_NAME_CHARS {
        return Err(format!(
            "The name can have at most {MAX_NAME_CHARS} characters."
        ));
    }
    if existing.any(|other| other == name) {
        return Err(format!("A snapshot named \"{name}\" exists already."));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
