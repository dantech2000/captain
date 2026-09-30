use std::os::unix::fs::symlink;

use super::remove_rc_block;
use crate::cli_tools::{RcFile, Shell, ToolPaths, with_block};

/// Switching to Manual does not write through a link, and says what to remove.
#[test]
fn removal_skips_a_linked_file_and_says_what_to_remove() {
    let home = std::env::temp_dir().join(format!("captain-install-{}", std::process::id()));
    std::fs::remove_dir_all(&home).ok();
    std::fs::create_dir_all(home.join("dotfiles")).unwrap();
    let text = with_block("export EDITOR=zed\n", &Shell::Zsh.block());
    let target = home.join("dotfiles/zshrc");
    std::fs::write(&target, &text).unwrap();
    let zsh = RcFile {
        shell: Shell::Zsh,
        path: home.join(".zshrc"),
    };
    symlink(&target, &zsh.path).unwrap();

    let paths = ToolPaths::new(&home, Some(home.join(".docker")));
    let error = remove_rc_block(&zsh, &paths).unwrap_err();
    assert!(
        error.starts_with("Captain did not change ~/.zshrc."),
        "{error}"
    );
    assert!(error.contains("# >>> captain >>>"), "{error}");
    assert_eq!(std::fs::read_to_string(&target).unwrap(), text);
    std::fs::remove_dir_all(&home).ok();
}
