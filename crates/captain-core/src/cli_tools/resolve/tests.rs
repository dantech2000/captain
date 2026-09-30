use std::path::{Path, PathBuf};

use super::{ToolSource, classify, parse_command_v};

#[test]
fn reads_paths_between_other_shell_output() {
    let output = "Welcome back\n/Users/dan/.rd/bin/docker\nalias helm=h\n/opt/homebrew/bin/helm\n";
    let found = parse_command_v(output, &["docker", "kubectl", "helm"]);
    assert_eq!(
        found,
        [
            (
                "docker".into(),
                Some(PathBuf::from("/Users/dan/.rd/bin/docker"))
            ),
            ("kubectl".into(), None),
            ("helm".into(), Some(PathBuf::from("/opt/homebrew/bin/helm"))),
        ]
    );
}

#[test]
fn names_the_source_by_path_or_link_target() {
    let home = Path::new("/Users/dan");
    let rd = Path::new("/Users/dan/.rd/bin/docker");
    let rd_app = Path::new(
        "/Applications/Rancher Desktop.app/Contents/Resources/resources/darwin/bin/docker",
    );
    assert_eq!(classify(rd, rd_app, home), ToolSource::RancherDesktop);
    let captain = Path::new("/Users/dan/.captain/bin/docker");
    assert_eq!(classify(captain, captain, home), ToolSource::Captain);
    let local = Path::new("/usr/local/bin/docker");
    let desktop = Path::new("/Applications/Docker.app/Contents/Resources/bin/docker");
    assert_eq!(classify(local, desktop, home), ToolSource::DockerDesktop);
    assert_eq!(
        classify(local, local, home),
        ToolSource::Other(local.into())
    );
}
