use super::*;

fn temp(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("captain-new-{name}-{}", std::process::id()));
    fs::remove_dir_all(&dir).ok();
    dir
}

#[test]
fn a_project_is_written_with_a_private_env_file() {
    let dir = temp("write");
    let files = [
        NewFile::new("compose.yaml", "services: {}\n"),
        NewFile::new("site/index.html", "<h1>Hi</h1>\n"),
        NewFile::private(".env", "PASSWORD=x\n"),
    ];

    write_project(&dir, &files).unwrap();

    assert_eq!(
        fs::read_to_string(dir.join("site/index.html")).unwrap(),
        "<h1>Hi</h1>\n"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(dir.join(".env")).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn a_folder_with_files_is_never_written() {
    let dir = temp("taken");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("compose.yaml"), "keep\n").unwrap();

    let result = write_project(&dir, &[NewFile::new("compose.yaml", "new\n")]);

    assert!(matches!(result, Err(NewProjectError::NotEmpty(_))));
    assert_eq!(
        fs::read_to_string(dir.join("compose.yaml")).unwrap(),
        "keep\n"
    );
    fs::remove_dir_all(&dir).ok();
}
