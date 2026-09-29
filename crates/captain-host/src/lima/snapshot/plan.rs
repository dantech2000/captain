//! Which files a snapshot holds, and where each one lives in the instance.

use std::path::{Path, PathBuf};

use super::swap::Replace;
use crate::lima::paths::LimaPaths;

/// One file of a snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotFile {
    /// The file name in the snapshot folder.
    pub name: &'static str,
    /// Where the file lives in the instance.
    pub live: PathBuf,
    /// A snapshot without it is not usable.
    pub required: bool,
}

/// The VM disk. On an instance made before Lima 2.1 it links to `diffdisk`.
pub const DISK: &str = "disk";

/// The files to copy for the instance in `paths`. Lima writes `cidata.iso`, the
/// logs, PID files, and sockets again on each start, so they are left out.
pub fn files(paths: &LimaPaths) -> Vec<SnapshotFile> {
    let instance = paths.instance_dir();
    let config = paths.config_dir();
    let file = |name, dir: &Path, required| SnapshotFile {
        name,
        live: dir.join(name),
        required,
    };
    vec![
        file(DISK, &instance, true),
        file("lima.yaml", &instance, true),
        file("lima-version", &instance, false),
        file("vz-efi", &instance, false),
        file("vz-identifier", &instance, false),
        file("user", &config, false),
        file("user.pub", &config, false),
        SnapshotFile {
            name: "captain-engine.yaml",
            live: paths.template_file(),
            required: false,
        },
    ]
}

/// The files of the snapshot in `dir` to put in place of `plan`. A required file
/// that the snapshot lacks fails the restore before it touches the instance; an
/// optional one is skipped.
pub fn restorable(plan: Vec<SnapshotFile>, dir: &Path) -> Result<Vec<Replace>, String> {
    let mut replace = Vec::new();
    for file in plan {
        let from = dir.join(file.name);
        if from.exists() {
            replace.push(Replace {
                from,
                live: file.live,
            });
        } else if file.required {
            return Err(format!(
                "The snapshot is damaged: {} is missing.",
                from.display()
            ));
        }
    }
    Ok(replace)
}

/// The file behind `live`: the link target for a legacy `disk` link, else `live`.
pub fn source_of(live: &Path) -> PathBuf {
    match std::fs::read_link(live) {
        Ok(target) => live.parent().map_or(target.clone(), |dir| dir.join(target)),
        Err(_) => live.to_path_buf(),
    }
}

#[cfg(test)]
mod tests;
