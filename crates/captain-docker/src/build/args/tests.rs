use std::path::{Path, PathBuf};

use captain_core::model::BuildSpec;

use super::*;

#[test]
fn every_field_becomes_an_argument() {
    let context = PathBuf::from("/src/app");
    let spec = BuildSpec {
        context: context.clone(),
        dockerfile: PathBuf::from("docker/Dockerfile.dev"),
        tag: "myapp:dev".into(),
        build_args: vec![("MSG".into(), "hi there".into())],
        target: Some("builder".into()),
    };
    let dockerfile = context.join(Path::new("docker/Dockerfile.dev"));
    let expected: Vec<String> = [
        "buildx",
        "build",
        "--builder",
        "default",
        "--progress",
        "plain",
        "--load",
        "-t",
        "myapp:dev",
        "-f",
        &dockerfile.display().to_string(),
        "--build-arg",
        "MSG=hi there",
        "--target",
        "builder",
        &context.display().to_string(),
    ]
    .map(String::from)
    .into();
    assert_eq!(build_args(&spec), expected);
}

#[test]
fn reads_the_buildx_version() {
    assert_eq!(
        parse_buildx_version("github.com/docker/buildx v0.35.0 a319e5b\n"),
        Some("v0.35.0".into())
    );
    assert_eq!(
        parse_buildx_version("docker: 'buildx' is not a docker command."),
        None
    );
}
