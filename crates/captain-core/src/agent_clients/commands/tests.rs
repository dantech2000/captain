use super::search_path;

#[test]
fn path_search_adds_each_extension_or_none() {
    let dir = std::env::temp_dir().join(format!("captain-commands-{}", std::process::id()));
    std::fs::remove_dir_all(&dir).ok();
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("claude.CMD"), "").unwrap();
    std::fs::write(dir.join("codex"), "").unwrap();
    let path = std::env::join_paths([dir.join("missing"), dir.clone()]).unwrap();
    let exts = [".EXE".to_string(), ".CMD".to_string()];
    assert_eq!(
        search_path(&["claude", "codex"], Some(&path), &exts),
        [
            ("claude".to_string(), Some(dir.join("claude.CMD"))),
            ("codex".to_string(), None),
        ]
    );
    assert_eq!(
        search_path(&["codex"], Some(&path), &[]),
        [("codex".to_string(), Some(dir.join("codex")))]
    );
    std::fs::remove_dir_all(&dir).ok();
}
