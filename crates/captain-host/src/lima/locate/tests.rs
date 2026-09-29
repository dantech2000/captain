use std::ffi::OsString;
use std::path::{Path, PathBuf};

use super::locate_limactl;

#[test]
fn bundled_copy_comes_first() {
    let exe = Path::new("/Applications/Captain.app/Contents/MacOS/captain");
    let path = OsString::from("/usr/bin:/opt/homebrew/bin");
    let found = locate_limactl(Some(exe), Some(&path), |_| true);
    let bundled = "/Applications/Captain.app/Contents/Resources/lima/bin/limactl";
    assert_eq!(found, Some(PathBuf::from(bundled)));
}
