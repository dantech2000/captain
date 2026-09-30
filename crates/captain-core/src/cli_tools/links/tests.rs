use std::path::{Path, PathBuf};

use super::{LinkState, ToolPaths, relink, remove_links, tool_links};
use crate::tools::Bundle;

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("captain-links-{name}-{}", std::process::id()));
    std::fs::remove_dir_all(&dir).ok();
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// A fake `Captain.app` in `dir` with every tool.
fn fake_bundle(dir: &Path) -> Bundle {
    let resources = dir.join("Captain.app/Contents/Resources");
    for tool in [
        "bin/docker",
        "bin/docker-credential-osxkeychain",
        "bin/captain",
        "bin/kubectl",
        "bin/helm",
        "cli-plugins/docker-compose",
        "cli-plugins/docker-buildx",
    ] {
        let path = resources.join(tool);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, "").unwrap();
    }
    Bundle::from_exe(&dir.join("Captain.app/Contents/MacOS/captain")).unwrap()
}

#[test]
fn links_every_tool_but_never_replaces_a_regular_file() {
    let dir = temp_dir("plan");
    let bundle = fake_bundle(&dir.join("Applications"));
    let paths = ToolPaths::new(&dir.join("home"), None);
    std::fs::create_dir_all(&paths.bin).unwrap();
    std::fs::write(paths.bin.join("captain"), "mine").unwrap();

    let reports = relink(&tool_links(&bundle, &paths));
    for report in &reports {
        assert_eq!(report.error, None);
        if report.link.path.ends_with("bin/captain") {
            assert_eq!(report.state, LinkState::NotALink);
        } else {
            assert_eq!(report.state, LinkState::Missing);
            assert_eq!(
                std::fs::read_link(&report.link.path).unwrap(),
                report.link.target
            );
        }
    }
    assert_eq!(
        std::fs::read_to_string(paths.bin.join("captain")).unwrap(),
        "mine"
    );
    assert_eq!(
        std::fs::read_link(paths.plugins.join("docker-buildx")).unwrap(),
        bundle.cli_plugins().join("docker-buildx")
    );
    assert_eq!(
        std::fs::read_link(paths.bin.join("helm")).unwrap(),
        bundle.bin().join("helm")
    );

    remove_links(&paths);
    assert!(!paths.bin.join("docker").exists());
    assert!(!paths.bin.join("kubectl").exists());
    assert!(paths.bin.join("captain").exists());
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn relinks_when_the_bundle_moves() {
    let dir = temp_dir("move");
    let paths = ToolPaths::new(&dir.join("home"), None);
    let old = fake_bundle(&dir.join("Downloads"));
    relink(&tool_links(&old, &paths));
    let new = fake_bundle(&dir.join("Applications"));

    let reports = relink(&tool_links(&new, &paths));
    let docker = reports
        .iter()
        .find(|report| report.link.path.ends_with("bin/docker"))
        .unwrap();
    assert_eq!(docker.state, LinkState::Stale(old.docker()));
    assert_eq!(
        std::fs::read_link(paths.bin.join("docker")).unwrap(),
        new.docker()
    );
    assert!(
        relink(&tool_links(&new, &paths))
            .iter()
            .all(|report| report.state == LinkState::Linked)
    );
    std::fs::remove_dir_all(&dir).ok();
}
