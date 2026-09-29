use std::path::{Path, PathBuf};

use super::{Replace, swap};

fn temp(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("captain-swap-{name}-{}", std::process::id()));
    std::fs::remove_dir_all(&dir).ok();
    std::fs::create_dir_all(dir.join("snap")).unwrap();
    std::fs::create_dir_all(dir.join("live")).unwrap();
    dir
}

fn copy(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::copy(from, to).map(drop)
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}

#[test]
fn replaces_existing_files_adds_missing_ones_and_cleans_up() {
    let dir = temp("ok");
    std::fs::write(dir.join("snap/disk"), "new disk").unwrap();
    std::fs::write(dir.join("snap/user"), "new key").unwrap();
    std::fs::write(dir.join("live/disk"), "old disk").unwrap();
    let files = [
        Replace {
            from: dir.join("snap/disk"),
            live: dir.join("live/disk"),
        },
        Replace {
            from: dir.join("snap/user"),
            live: dir.join("live/_config/user"),
        },
    ];
    swap(&files, &dir.join("live"), &copy).expect("swaps");
    assert_eq!(read(&dir.join("live/disk")), "new disk");
    assert_eq!(read(&dir.join("live/_config/user")), "new key");
    assert_eq!(read(&dir.join("snap/disk")), "new disk");
    assert!(!dir.join("live/.restore-staging").exists());
    assert!(!dir.join("live/.restore-backup").exists());
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn a_failed_rename_puts_every_old_file_back() {
    let dir = temp("rollback");
    std::fs::write(dir.join("snap/disk"), "new disk").unwrap();
    std::fs::write(dir.join("snap/efi"), "new efi").unwrap();
    std::fs::write(dir.join("live/disk"), "old disk").unwrap();
    std::fs::write(dir.join("live/efi"), "old efi").unwrap();
    let files = [
        Replace {
            from: dir.join("snap/disk"),
            live: dir.join("live/disk"),
        },
        Replace {
            from: dir.join("snap/efi"),
            live: dir.join("live/efi"),
        },
    ];
    // The staged `efi` never appears, so its rename fails after `disk` moved.
    let result = swap(&files, &dir.join("live"), &|from, to| {
        if from.ends_with("efi") {
            Ok(())
        } else {
            copy(from, to)
        }
    });
    assert!(result.is_err());
    assert_eq!(read(&dir.join("live/disk")), "old disk");
    assert_eq!(read(&dir.join("live/efi")), "old efi");
    std::fs::remove_dir_all(&dir).ok();
}
