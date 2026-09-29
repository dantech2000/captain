//! Builds the `docker buildx build` arguments. Pure, so tests need no CLI.

use captain_core::model::BuildSpec;

/// `buildx build` with plain progress for `spec`. `--builder default` is the `docker`
/// driver on `DOCKER_HOST`, so the image lands in Captain's engine even when the
/// user picked another builder. `--load` keeps the image local with any driver.
pub fn build_args(spec: &BuildSpec) -> Vec<String> {
    let mut args: Vec<String> = [
        "buildx",
        "build",
        "--builder",
        "default",
        "--progress",
        "plain",
        "--load",
        "-t",
        &spec.tag,
        "-f",
    ]
    .map(String::from)
    .into();
    args.push(spec.dockerfile_path().display().to_string());
    for (key, value) in &spec.build_args {
        args.push("--build-arg".into());
        args.push(format!("{key}={value}"));
    }
    if let Some(target) = &spec.target {
        args.push("--target".into());
        args.push(target.clone());
    }
    args.push(spec.context.display().to_string());
    args
}

/// `spec` with an absolute context. The CLI runs in the context folder, so a
/// relative context in the arguments would point inside itself.
pub fn with_absolute_context(spec: &BuildSpec) -> BuildSpec {
    let mut spec = spec.clone();
    if let Ok(context) = std::path::absolute(&spec.context) {
        spec.context = context;
    }
    spec
}

/// The version from `docker buildx version`, which prints
/// `github.com/docker/buildx v0.35.0 a319e5b`.
pub fn parse_buildx_version(stdout: &str) -> Option<String> {
    let version = stdout.split_whitespace().nth(1)?;
    version.starts_with('v').then(|| version.to_string())
}

#[cfg(test)]
mod tests;
