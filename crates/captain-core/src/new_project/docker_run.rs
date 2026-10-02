//! Turns a pasted `docker run` command into one Compose service, flag by flag,
//! the way composerize does. Flags Captain does not convert become warnings.
//! See the Paste a docker run command section of docs/features/0040-new-projects.md.

mod flags;
mod words;

use flags::Flag;
use words::split_words;

use super::project_name::{image_project_name, to_project_name};
use super::{ServiceSpec, is_windows_absolute, named_volume};

/// A `docker run` command as one Compose service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunConversion {
    /// The service name: `--name`, else the image's repository name, valid for
    /// Compose.
    pub service_name: String,
    /// The `--name` value, for the New sheet's project name.
    pub container_name: Option<String>,
    pub service: ServiceSpec,
    /// One sentence for each thing Captain left out or changed.
    pub warnings: Vec<String>,
}

/// Converts the `docker run` command in `text`. It may start with `docker run`,
/// `docker container run`, `podman run`, or with the flags.
pub fn convert_docker_run(text: &str) -> Result<RunConversion, String> {
    let words = split_words(text)?;
    let args = run_arguments(&words)?;
    let mut builder = Builder::default();
    let mut rest = args.iter();
    let mut image = None;
    while let Some(word) = rest.next() {
        if word == "--" {
            image = rest.next();
            break;
        }
        if let Some(long) = word.strip_prefix("--") {
            builder.long(long, &mut rest)?;
        } else if let Some(letters) = word.strip_prefix('-').filter(|l| !l.is_empty()) {
            builder.short(letters, &mut rest)?;
        } else {
            image = Some(word);
            break;
        }
    }
    let Some(image) = image else {
        return Err("The command names no image. Add the image after the flags.".into());
    };
    builder.service.image = image.clone();
    builder.service.command = rest.cloned().collect();
    let container_name = builder.service.container_name.clone();
    let service_name = match &container_name {
        Some(name) => to_project_name(name),
        None => image_project_name(image),
    };
    Ok(RunConversion {
        service_name,
        container_name,
        service: builder.service,
        warnings: builder.warnings,
    })
}

/// The words after `docker run`, `docker container run`, or `podman run`. A
/// leading `$` prompt and `sudo` are dropped. Words that start with a flag are
/// taken as they are.
fn run_arguments(words: &[String]) -> Result<&[String], String> {
    let mut words = words;
    while words.first().is_some_and(|w| w == "$" || w == "sudo") {
        words = &words[1..];
    }
    let Some(first) = words.first() else {
        return Err("Paste a docker run command.".into());
    };
    if first != "docker" && first != "podman" {
        return Ok(words);
    }
    let after: Vec<&str> = words[1..].iter().take(2).map(String::as_str).collect();
    match after.as_slice() {
        ["run", ..] => Ok(&words[2..]),
        ["container", "run"] => Ok(&words[3..]),
        _ => Err(format!(
            "Captain converts docker run commands, and this is {first} {}.",
            after.first().copied().unwrap_or_default()
        )),
    }
}

/// The service so far, and the warnings.
#[derive(Default)]
struct Builder {
    service: ServiceSpec,
    warnings: Vec<String>,
    /// Flags already warned about, so each is named once.
    warned: Vec<String>,
}

impl Builder {
    /// `--name value`, `--name=value`, or a flag without a value.
    fn long<'a>(
        &mut self,
        text: &str,
        rest: &mut impl Iterator<Item = &'a String>,
    ) -> Result<(), String> {
        let (name, attached) = match text.split_once('=') {
            Some((name, value)) => (name, Some(value.to_string())),
            None => (text, None),
        };
        let flag = Flag::long(name);
        let shown = format!("--{name}");
        if flag == Flag::Unknown && attached.is_none() {
            return Err(unknown_flag(&shown));
        }
        if !flag.takes_value() {
            // `--rm=false` and the like turn the flag off.
            if attached.as_deref() != Some("false") {
                self.apply(flag, &shown, None);
            }
            return Ok(());
        }
        let value = match attached {
            Some(value) => value,
            None => next_value(&shown, rest)?,
        };
        self.apply(flag, &shown, Some(value));
        Ok(())
    }

    /// `-it`, `-p 80:80`, or `-p80:80`: letters until one takes a value, which
    /// is the rest of the word or the next word.
    fn short<'a>(
        &mut self,
        letters: &str,
        rest: &mut impl Iterator<Item = &'a String>,
    ) -> Result<(), String> {
        for (at, letter) in letters.char_indices() {
            let flag = Flag::short(letter);
            let shown = format!("-{letter}");
            if flag == Flag::Unknown {
                return Err(unknown_flag(&shown));
            }
            if !flag.takes_value() {
                self.apply(flag, &shown, None);
                continue;
            }
            let attached = &letters[at + letter.len_utf8()..];
            let value = match attached.strip_prefix('=').unwrap_or(attached) {
                "" => next_value(&shown, rest)?,
                value => value.to_string(),
            };
            self.apply(flag, &shown, Some(value));
            return Ok(());
        }
        Ok(())
    }

    /// Adds `flag` with `value` to the service. `shown` is the flag as typed.
    fn apply(&mut self, flag: Flag, shown: &str, value: Option<String>) {
        let s = &mut self.service;
        let Some(value) = value else {
            match flag {
                Flag::Interactive => s.stdin_open = true,
                Flag::Tty => s.tty = true,
                Flag::Rm => self.warn_once(
                    "--rm",
                    "Compose has no --rm, so the container stays after it stops. \
                     docker compose down removes it.",
                ),
                Flag::Detach => {}
                _ => self.unconverted(shown),
            }
            return;
        };
        match flag {
            Flag::Publish => s.ports.push(value),
            Flag::Env => match value.split_once('=') {
                Some((key, val)) => s.environment.push((key.into(), Some(val.into()))),
                None => {
                    self.warnings.push(format!(
                        "{value} has no value, so it comes from the shell that runs docker compose."
                    ));
                    self.service.environment.push((value, None));
                }
            },
            Flag::EnvFile => {
                if is_relative(&value) {
                    self.warn_relative(&value, "--env-file");
                }
                self.service.env_file.push(value);
            }
            Flag::Volume => {
                let source = value.split_once(':').map(|(source, _)| source);
                if named_volume(&value).is_none() && source.is_some_and(is_relative) {
                    self.warn_relative(source.unwrap_or_default(), "-v");
                }
                self.service.volumes.push(value);
            }
            Flag::Name => s.container_name = Some(value),
            Flag::Restart => s.restart = (value != "no").then_some(value),
            Flag::Network => match value.as_str() {
                "host" | "none" | "bridge" => s.network_mode = Some(value),
                _ if value.starts_with("container:") => s.network_mode = Some(value),
                _ => s.networks.push(value),
            },
            Flag::Workdir => s.working_dir = Some(value),
            Flag::Entrypoint => s.entrypoint = vec![value],
            Flag::User => s.user = Some(value),
            Flag::Hostname => s.hostname = Some(value),
            Flag::Label => {
                let (key, val) = value.split_once('=').unwrap_or((&value, ""));
                s.labels.push((key.into(), val.into()));
            }
            Flag::Memory => s.mem_limit = Some(value),
            Flag::Cpus => s.cpus = Some(value),
            Flag::Platform => s.platform = Some(value),
            Flag::Pull if matches!(value.as_str(), "always" | "missing" | "never") => {
                s.pull_policy = Some(value)
            }
            _ => self.unconverted(shown),
        }
    }

    /// A relative path meant the folder where the command ran; in the project
    /// it means the project folder.
    fn warn_relative(&mut self, path: &str, flag: &str) {
        self.warnings.push(format!(
            "{path} in {flag} was relative to the folder where you ran docker run. \
             In the project it is inside the project folder. Copy the files there, \
             or write the full path in compose.yaml."
        ));
    }

    fn unconverted(&mut self, flag: &str) {
        self.warn_once(
            flag,
            &format!("Captain did not convert {flag}. Add it to compose.yaml by hand."),
        );
    }

    fn warn_once(&mut self, flag: &str, warning: &str) {
        if !self.warned.iter().any(|known| known == flag) {
            self.warned.push(flag.to_string());
            self.warnings.push(warning.to_string());
        }
    }
}

/// The error for a flag the `docker run` reference does not have.
fn unknown_flag(flag: &str) -> String {
    format!(
        "Captain does not know {flag}, so it cannot tell if the next word is its value. \
         Write it as {flag}=value, or remove it."
    )
}

/// True for a path that is not absolute: not `/`, `~`, `$`, `C:\`, or
/// `\\server`.
fn is_relative(path: &str) -> bool {
    !(path.starts_with(['/', '~', '$']) || is_windows_absolute(path))
}

/// The word after `flag`, which takes a value.
fn next_value<'a>(
    flag: &str,
    rest: &mut impl Iterator<Item = &'a String>,
) -> Result<String, String> {
    rest.next()
        .cloned()
        .ok_or_else(|| format!("{flag} needs a value after it."))
}

#[cfg(test)]
mod tests;
