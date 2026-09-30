use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};

use super::{
    RcAccess, RcFile, RcState, Shell, add_block, rc_access_in, rc_files, rc_state, remove_block,
};

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("captain-rc-{name}-{}", std::process::id()));
    std::fs::remove_dir_all(&dir).ok();
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn skipped(access: RcAccess) -> bool {
    matches!(access, RcAccess::Skip(_))
}

#[test]
fn picks_the_files_of_the_shells_that_exist() {
    let home = temp_dir("pick");
    std::fs::write(home.join(".bashrc"), "").unwrap();
    std::fs::write(home.join(".bash_profile"), "").unwrap();
    let files = rc_files(&home, Some(Path::new("/bin/zsh")));
    let expected = [
        (Shell::Zsh, home.join(".zshrc")),
        (Shell::Bash, home.join(".bash_profile")),
    ]
    .map(|(shell, path)| RcFile { shell, path });
    assert_eq!(files, expected);

    std::fs::create_dir_all(home.join(".config/fish")).unwrap();
    let fish = rc_files(&home, None).pop().unwrap();
    assert_eq!(fish.path, home.join(".config/fish/conf.d/captain.fish"));
    std::fs::remove_dir_all(&home).ok();
}

#[test]
fn skips_links_into_the_nix_store_read_only_and_managed_files() {
    let dir = temp_dir("skip");
    let store = dir.join("nix/store");
    std::fs::create_dir_all(&store).unwrap();
    std::fs::write(store.join("hm-zshrc"), "").unwrap();
    let linked = dir.join(".zshrc");
    symlink(store.join("hm-zshrc"), &linked).unwrap();
    assert_eq!(
        rc_access_in(&linked, &store, |_| false),
        RcAccess::Skip("It links into /nix/store (home-manager or Nix).".into())
    );

    let plain = dir.join(".bashrc");
    std::fs::write(&plain, "").unwrap();
    assert_eq!(rc_access_in(&plain, &store, |_| false), RcAccess::Writable);
    assert!(skipped(rc_access_in(&plain, &store, |_| true)));
    std::fs::set_permissions(&plain, std::fs::Permissions::from_mode(0o444)).unwrap();
    assert!(skipped(rc_access_in(&plain, &store, |_| false)));

    let missing = dir.join("new/conf.d/captain.fish");
    assert_eq!(
        rc_access_in(&missing, &store, |_| false),
        RcAccess::Writable
    );
    assert!(skipped(rc_access_in(
        &store.join("fish/captain.fish"),
        &store,
        |_| false
    )));
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn adds_and_removes_the_block_and_the_own_fish_file() {
    let dir = temp_dir("edit");
    let zsh = RcFile {
        shell: Shell::Zsh,
        path: dir.join(".zshrc"),
    };
    std::fs::write(&zsh.path, "export EDITOR=zed\n").unwrap();
    add_block(&zsh).unwrap();
    add_block(&zsh).unwrap();
    assert_eq!(rc_state(&zsh.path), RcState::Added);
    assert_eq!(remove_block(&zsh), Ok(true));
    assert_eq!(
        std::fs::read_to_string(&zsh.path).unwrap(),
        "export EDITOR=zed\n"
    );

    let fish = RcFile {
        shell: Shell::Fish,
        path: dir.join("fish/conf.d/captain.fish"),
    };
    add_block(&fish).unwrap();
    assert_eq!(remove_block(&fish), Ok(true));
    assert!(!fish.path.exists());
    std::fs::remove_dir_all(&dir).ok();
}

/// A write replaces the file whole, keeps its permissions, and backs up the
/// user's version once.
#[test]
fn writes_keep_the_mode_and_back_up_the_first_version_once() {
    let dir = temp_dir("atomic");
    let zsh = RcFile {
        shell: Shell::Zsh,
        path: dir.join(".zshrc"),
    };
    std::fs::write(&zsh.path, "export EDITOR=zed\n").unwrap();
    std::fs::set_permissions(&zsh.path, std::fs::Permissions::from_mode(0o640)).unwrap();
    add_block(&zsh).unwrap();
    assert_eq!(remove_block(&zsh), Ok(true));
    add_block(&zsh).unwrap();

    let mode = std::fs::metadata(&zsh.path).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o640);
    assert_eq!(
        std::fs::read_to_string(dir.join(".zshrc.captain-backup")).unwrap(),
        "export EDITOR=zed\n"
    );
    let names: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(names.len(), 2, "{names:?}");
    std::fs::remove_dir_all(&dir).ok();
}
