use std::fs;

use captain_core::extension::MANIFEST_FILE;

use super::Swap;

#[test]
fn a_roll_back_restores_the_old_files_and_drops_the_new_ones() {
    let root = std::env::temp_dir().join(format!("captain-ext-swap-{}", std::process::id()));
    let (live, staged, backup) = (root.join("live"), root.join("staged"), root.join("backup"));
    fs::create_dir_all(live.join("ui")).unwrap();
    fs::create_dir_all(live.join("compose")).unwrap();
    fs::write(live.join("ui/old.html"), "old").unwrap();
    fs::write(live.join("compose/old.json"), "old").unwrap();
    fs::write(live.join(MANIFEST_FILE), "old").unwrap();
    fs::write(live.join("data.txt"), "kept").unwrap();
    fs::create_dir_all(staged.join("ui")).unwrap();
    fs::create_dir_all(staged.join("bin")).unwrap();
    fs::write(staged.join("ui/new.html"), "new").unwrap();

    let mut swap = Swap::new(live.clone(), staged, backup.clone());
    swap.replace("new").unwrap();
    assert!(live.join("ui/new.html").is_file() && live.join("bin").is_dir());
    assert_eq!(fs::read_to_string(live.join(MANIFEST_FILE)).unwrap(), "new");
    // The new backend writes its Compose file.
    fs::create_dir_all(live.join("compose")).unwrap();
    swap.roll_back().unwrap();

    let restored = (
        live.join("ui/old.html").is_file(),
        live.join("compose/old.json").is_file(),
        live.join("ui/new.html").exists(),
        live.join("bin").exists(),
        fs::read_to_string(live.join(MANIFEST_FILE)).unwrap(),
        live.join("data.txt").is_file(),
        backup.exists(),
    );
    fs::remove_dir_all(&root).ok();
    assert_eq!(
        restored,
        (true, true, false, false, "old".into(), true, false)
    );
}
