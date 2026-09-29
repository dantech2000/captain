use std::path::PathBuf;

use thiserror::Error;

use crate::model::{BuildSpec, ImageReference};

/// The Dockerfile name when the field is empty.
pub const DEFAULT_DOCKERFILE: &str = "Dockerfile";

/// The Build dialog's input, as the user typed it. [`BuildForm::to_spec`] checks it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BuildForm {
    /// The context folder.
    pub context: String,
    /// The Dockerfile, relative to the context or absolute. Empty means `Dockerfile`.
    pub dockerfile: String,
    pub tag: String,
    /// `KEY=value` lines. Empty lines are skipped.
    pub build_args: Vec<String>,
    /// The stage to build. Empty builds the last stage.
    pub target: String,
}

/// Why the Build dialog's input is not valid.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BuildFormError {
    #[error("Choose a folder to build")]
    NoContext,
    #[error("Enter a tag, for example myapp:dev")]
    NoTag,
    #[error("\"{0}\" is not a valid tag")]
    Tag(String),
    #[error("Build argument \"{0}\" is not KEY=value")]
    BuildArg(String),
    #[error("The target cannot contain spaces")]
    Target,
}

impl BuildForm {
    /// Checks the input and builds the spec. Returns the first problem it finds.
    pub fn to_spec(&self) -> Result<BuildSpec, BuildFormError> {
        let context = self.context.trim();
        if context.is_empty() {
            return Err(BuildFormError::NoContext);
        }
        let tag = self.tag.trim();
        if tag.is_empty() {
            return Err(BuildFormError::NoTag);
        }
        let reference = ImageReference::parse(tag)
            .filter(|reference| !reference.tag.is_empty())
            .ok_or_else(|| BuildFormError::Tag(tag.to_string()))?;
        let build_args = self
            .build_args
            .iter()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .map(|line| match line.split_once('=') {
                Some((key, value)) if !key.is_empty() && !key.contains(char::is_whitespace) => {
                    Ok((key.to_string(), value.to_string()))
                }
                _ => Err(BuildFormError::BuildArg(line.to_string())),
            })
            .collect::<Result<Vec<_>, _>>()?;
        let target = self.target.trim();
        if target.contains(char::is_whitespace) {
            return Err(BuildFormError::Target);
        }
        let dockerfile = match self.dockerfile.trim() {
            "" => DEFAULT_DOCKERFILE,
            name => name,
        };
        Ok(BuildSpec {
            context: PathBuf::from(context),
            dockerfile: PathBuf::from(dockerfile),
            tag: reference.to_string(),
            build_args,
            target: (!target.is_empty()).then(|| target.to_string()),
        })
    }
}

#[cfg(test)]
mod tests;
