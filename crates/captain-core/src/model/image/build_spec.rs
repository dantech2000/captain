use std::path::PathBuf;

/// What `docker buildx build` needs. Build it with
/// [`BuildForm`](crate::store::BuildForm), which checks the user's input.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BuildSpec {
    /// The build context folder.
    pub context: PathBuf,
    /// The Dockerfile. A relative path is relative to the context.
    pub dockerfile: PathBuf,
    /// The reference to tag the result with, for example `myapp:dev`.
    pub tag: String,
    /// `--build-arg` pairs, in the order the user typed them.
    pub build_args: Vec<(String, String)>,
    /// The stage to build, for a multi-stage Dockerfile.
    pub target: Option<String>,
}

impl BuildSpec {
    /// The Dockerfile path, joined to the context when it is relative.
    pub fn dockerfile_path(&self) -> PathBuf {
        self.context.join(&self.dockerfile)
    }
}
