use super::captain_command;

#[test]
fn a_development_build_finds_the_cli_next_to_it() {
    let dir = std::env::temp_dir().join(format!("captain-paths-{}", std::process::id()));
    std::fs::remove_dir_all(&dir).ok();
    std::fs::create_dir_all(&dir).unwrap();
    let cli = dir.join(format!("captain-cli{}", std::env::consts::EXE_SUFFIX));
    std::fs::write(&cli, "").unwrap();
    let exe = dir.join(format!("captain{}", std::env::consts::EXE_SUFFIX));
    assert_eq!(captain_command(&dir.join("home"), &exe), Some(cli));
    std::fs::remove_dir_all(&dir).ok();
}
