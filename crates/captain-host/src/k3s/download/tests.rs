use captain_core::kubernetes::{K3sAssets, K3sVersion};

use super::{ensure, sha256};
use crate::cancel::Cancel;

#[test]
fn hashes_a_file() {
    let file = std::env::temp_dir().join(format!("captain-sha-{}", std::process::id()));
    std::fs::write(&file, "abc").unwrap();
    assert_eq!(
        sha256(&file).unwrap(),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    std::fs::remove_file(&file).ok();
}

#[test]
fn a_complete_folder_needs_no_download() {
    let cache = std::env::temp_dir().join(format!("captain-k3s-cache-{}", std::process::id()));
    let version: K3sVersion = "v1.36.4+k3s1".parse().unwrap();
    let assets = K3sAssets::for_arch("aarch64").unwrap();
    let folder = cache.join(version.as_str());
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::write(folder.join(assets.binary), "").unwrap();
    std::fs::write(folder.join(assets.images), "").unwrap();
    let mut lines = Vec::new();
    let found = ensure(&cache, &version, &assets, &Cancel::default(), &mut |line| {
        lines.push(line)
    })
    .unwrap();
    assert_eq!(found, folder);
    assert!(lines.is_empty());
    std::fs::remove_dir_all(&cache).ok();
}
