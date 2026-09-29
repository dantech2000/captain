use std::path::Path;

use super::{DISK, files, source_of};
use crate::lima::paths::LimaPaths;

#[test]
fn copies_the_disk_config_and_keys_but_not_the_runtime_files() {
    let paths = LimaPaths::for_home(Path::new("/Users/ada"));
    let plan = files(&paths);
    let names: Vec<_> = plan.iter().map(|file| file.name).collect();
    assert_eq!(
        names,
        [
            "disk",
            "lima.yaml",
            "lima-version",
            "vz-efi",
            "vz-identifier",
            "user",
            "user.pub",
            "captain-engine.yaml"
        ]
    );
    let required: Vec<_> = plan.iter().filter(|f| f.required).map(|f| f.name).collect();
    assert_eq!(required, ["disk", "lima.yaml"]);
    assert_eq!(
        plan[5].live,
        Path::new("/Users/ada/.captain/lima/_config/user")
    );
    assert_eq!(
        plan[7].live,
        Path::new("/Users/ada/.captain/captain-engine.yaml")
    );
}

#[cfg(unix)]
#[test]
fn a_legacy_disk_link_resolves_to_diffdisk() {
    let dir = std::env::temp_dir().join(format!("captain-plan-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("diffdisk"), b"x").unwrap();
    std::os::unix::fs::symlink("diffdisk", dir.join(DISK)).unwrap();
    assert_eq!(source_of(&dir.join(DISK)), dir.join("diffdisk"));
    assert_eq!(source_of(&dir.join("diffdisk")), dir.join("diffdisk"));
    std::fs::remove_dir_all(&dir).ok();
}
