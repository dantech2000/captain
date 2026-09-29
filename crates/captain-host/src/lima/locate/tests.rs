use std::ffi::OsString;
use std::path::{Path, PathBuf};

use super::{bundled, locate_limactl};

const EXE: &str = "/Applications/Captain.app/Contents/MacOS/captain";
const BUNDLED: &str = "/Applications/Captain.app/Contents/Resources/lima/bin/limactl";

#[test]
fn bundled_copy_sits_in_resources() {
    assert_eq!(bundled(Path::new(EXE)), Some(PathBuf::from(BUNDLED)));
}

#[test]
fn bundled_copy_comes_first() {
    let path = OsString::from("/usr/bin:/opt/homebrew/bin");
    let found = locate_limactl(Some(Path::new(EXE)), Some(&path), |_| true);
    assert_eq!(found, Some(PathBuf::from(BUNDLED)));
}

#[test]
fn path_comes_before_homebrew() {
    let path = OsString::from("/custom/bin");
    let found = locate_limactl(Some(Path::new(EXE)), Some(&path), |p| {
        p == Path::new("/custom/bin/limactl") || p == Path::new("/opt/homebrew/bin/limactl")
    });
    assert_eq!(found, Some(PathBuf::from("/custom/bin/limactl")));
}

#[test]
fn falls_back_to_homebrew() {
    let found = locate_limactl(None, None, |p| p == Path::new("/usr/local/bin/limactl"));
    assert_eq!(found, Some(PathBuf::from("/usr/local/bin/limactl")));
}

#[test]
fn nothing_found() {
    assert_eq!(locate_limactl(Some(Path::new(EXE)), None, |_| false), None);
}
