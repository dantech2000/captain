use std::path::PathBuf;

use super::{BINARY, locate_docker};

#[test]
fn falls_back_to_the_home_install_folders() {
    let home = PathBuf::from("/home/dan");
    let wanted = home.join(".docker/bin").join(BINARY);
    let path = std::env::join_paths(["/nowhere"]).expect("join paths");
    let found = locate_docker(None, Some(&path), Some(&home), |p| p == wanted);
    assert_eq!(found, Some(wanted));
}
