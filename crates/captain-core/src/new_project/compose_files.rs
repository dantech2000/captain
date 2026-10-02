//! The files of a project made with Run an image or Paste a docker run command.
//! Values that look secret go in `.env`, like the templates' passwords.

use super::compose_text::last_wins;
use super::{ComposeDoc, NewFile};
use crate::model::is_secret_key;

/// `compose.yaml` for `doc`. An environment value whose name looks secret
/// ([`is_secret_key`]), such as `POSTGRES_PASSWORD`, goes in a private `.env`
/// instead, and `compose.yaml` reads it with `${NAME:?...}`. A `.gitignore` then
/// keeps `.env` out of Git. Other values stay in `compose.yaml`.
pub fn project_files(mut doc: ComposeDoc) -> Result<Vec<NewFile>, String> {
    let mut secrets: Vec<(String, String)> = Vec::new();
    for (_, service) in &mut doc.services {
        // A later value wins, as it does in the service, also an empty one or one
        // that passes the shell's value. Only the value that wins can be a secret.
        let found: Vec<(String, String)> = last_wins(&service.environment)
            .into_iter()
            .filter_map(|(name, value)| {
                let value = value.as_deref()?;
                // `${NAME:?...}` refuses an empty value, so an empty one stays.
                (!value.is_empty() && is_secret_key(name) && is_variable_name(name))
                    .then(|| (name.clone(), value.to_string()))
            })
            .collect();
        for (name, value) in found {
            service.dotenv.push(name.clone());
            match secrets.iter().find(|(known, _)| *known == name) {
                Some((_, known)) if *known != value => {
                    return Err(format!("{name} has two values. Keep one."));
                }
                Some(_) => {}
                None => secrets.push((name, value)),
            }
        }
    }
    let mut files = vec![NewFile::new("compose.yaml", doc.to_yaml())];
    if secrets.is_empty() {
        return Ok(files);
    }
    let mut env = String::from(
        "# Secrets for this project. Compose reads this file. Keep it out of version control.\n",
    );
    for (name, value) in &secrets {
        env.push_str(&format!("{name}={}\n", dotenv_value(name, value)?));
    }
    files.push(NewFile::private(".env", env));
    files.push(NewFile::new(".gitignore", ".env\n"));
    Ok(files)
}

/// A name Compose can interpolate: a letter or `_`, then letters, digits, and
/// `_`.
fn is_variable_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// `value` in single quotes, which `.env` reads literally, without `$`
/// interpolation
/// (<https://docs.docker.com/compose/how-tos/environment-variables/variable-interpolation/#env-file-syntax>).
/// A quote, a backslash, or a line break would need escapes, so Captain refuses
/// them.
fn dotenv_value(name: &str, value: &str) -> Result<String, String> {
    if value.contains(['\'', '\\', '\n', '\r']) {
        return Err(format!(
            "{name}: Captain cannot write a value with ', \\, or a line break to .env. \
             Leave it empty here and set it in .env after Captain creates the project."
        ));
    }
    Ok(format!("'{value}'"))
}

#[cfg(test)]
mod tests;
