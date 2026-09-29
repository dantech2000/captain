use std::path::{Path, PathBuf};

use super::{Bundle, locate_tool};

const EXE: &str = "/Applications/Captain.app/Contents/MacOS/captain";

fn bundle() -> Bundle {
    Bundle::from_exe(Path::new(EXE)).expect("an app bundle")
}

fn path_list(dirs: &[&str]) -> std::ffi::OsString {
    std::env::join_paths(dirs).expect("join paths")
}

#[test]
fn tools_sit_in_resources() {
    let resources = Path::new("/Applications/Captain.app/Contents/Resources");
    let bundle = bundle();
    assert_eq!(bundle.limactl(), resources.join("lima/bin/limactl"));
    assert_eq!(bundle.docker(), resources.join("bin/docker"));
    assert_eq!(bundle.cli_plugins(), resources.join("cli-plugins"));
}

#[test]
fn a_cargo_build_has_no_bundle() {
    assert_eq!(
        Bundle::from_exe(Path::new("/src/captain/target/debug/captain")),
        None
    );
}

#[test]
fn bundled_copy_comes_first() {
    let path = path_list(&["/usr/bin"]);
    let found = locate_tool("docker", Some(bundle().docker()), Some(&path), [], |_| true);
    assert_eq!(found, Some(bundle().docker()));
}

#[test]
fn path_comes_before_fallback_dirs() {
    let path = path_list(&["/nowhere", "/custom/bin"]);
    let wanted = Path::new("/custom/bin").join("docker");
    let fallback = [PathBuf::from("/usr/local/bin")];
    let found = locate_tool("docker", None, Some(&path), fallback, |p| {
        p == wanted || p.starts_with("/usr/local/bin")
    });
    assert_eq!(found, Some(wanted));
}

#[test]
fn falls_back_when_the_bundled_copy_is_missing() {
    let wanted = Path::new("/opt/homebrew/bin").join("limactl");
    let fallback = [PathBuf::from("/opt/homebrew/bin")];
    let found = locate_tool("limactl", Some(bundle().limactl()), None, fallback, |p| {
        p == wanted
    });
    assert_eq!(found, Some(wanted));
}

#[test]
fn the_cli_in_resources_bin_finds_the_same_bundle() {
    let cli = Path::new("/Applications/Captain.app/Contents/Resources/bin/captain");
    assert_eq!(Bundle::from_exe(cli), Some(bundle()));
}
