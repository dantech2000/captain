use std::path::{Path, PathBuf};

use super::{update, with_plugin_dir, without_plugin_dir};

const DIR: &str = "/Users/dan/.captain/cli-plugins";

#[test]
fn adds_the_folder_first_and_keeps_other_keys_in_order() {
    let user = r#"{"auths":{"ghcr.io":{}},"credsStore":"osxkeychain","cliPluginsExtraDirs":["/opt/plugins"],"currentContext":"rancher-desktop"}"#;
    let text = with_plugin_dir(Some(user), Path::new(DIR))
        .unwrap()
        .unwrap();
    assert_eq!(
        text,
        "{\n\t\"auths\": {\n\t\t\"ghcr.io\": {}\n\t},\n\t\"credsStore\": \"osxkeychain\",\n\t\
         \"cliPluginsExtraDirs\": [\n\t\t\"/Users/dan/.captain/cli-plugins\",\n\t\t\"/opt/plugins\"\n\t],\n\t\
         \"currentContext\": \"rancher-desktop\"\n}\n"
    );
    assert_eq!(with_plugin_dir(Some(&text), Path::new(DIR)), Ok(None));

    let removed = without_plugin_dir(&text, Path::new(DIR)).unwrap().unwrap();
    assert!(removed.contains("\"/opt/plugins\""));
    assert!(!removed.contains(DIR));
}

#[test]
fn removing_the_last_folder_removes_the_key_and_bad_json_is_an_error() {
    let text = with_plugin_dir(None, Path::new(DIR)).unwrap().unwrap();
    assert_eq!(
        without_plugin_dir(&text, Path::new(DIR)).unwrap().unwrap(),
        "{}\n"
    );
    assert!(with_plugin_dir(Some("{ not json"), Path::new(DIR)).is_err());
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("captain-{name}-{}", std::process::id()));
    std::fs::remove_dir_all(&dir).ok();
    std::fs::create_dir_all(dir.join(".captain")).unwrap();
    dir
}

#[cfg(unix)]
#[test]
fn writes_through_a_link_and_backs_up_once() {
    use super::add_plugin_dir;

    let dir = temp_dir("plugin-config");
    let plugins = dir.join(".captain/cli-plugins");
    std::fs::create_dir_all(dir.join("dotfiles")).unwrap();
    let target = dir.join("dotfiles/config.json");
    std::fs::write(&target, r#"{"credsStore":"osxkeychain"}"#).unwrap();
    let link = dir.join("config.json");
    std::os::unix::fs::symlink(&target, &link).unwrap();

    assert_eq!(add_plugin_dir(&link, &plugins), Ok(true));
    assert!(
        std::fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    let text = std::fs::read_to_string(&target).unwrap();
    assert!(text.contains(&plugins.display().to_string()));
    let backup = dir.join("config.json.captain-backup");
    assert_eq!(
        std::fs::read_to_string(&backup).unwrap(),
        r#"{"credsStore":"osxkeychain"}"#
    );

    assert_eq!(add_plugin_dir(&link, &dir.join(".captain/other")), Ok(true));
    assert_eq!(
        std::fs::read_to_string(&backup).unwrap(),
        r#"{"credsStore":"osxkeychain"}"#
    );
    std::fs::remove_dir_all(&dir).ok();
}

/// A `docker login` that writes the file while Captain merges is kept: Captain
/// sees the change before its rename and merges again.
#[test]
fn a_change_by_another_program_meanwhile_is_kept() {
    let dir = temp_dir("plugin-race");
    let plugins = dir.join(".captain/cli-plugins");
    let config = dir.join("config.json");
    std::fs::write(&config, "{}").unwrap();
    let login = r#"{"auths":{"ghcr.io":{}}}"#;
    let mut logins = 0;
    let changed = update(
        &config,
        &plugins,
        |text| with_plugin_dir(text, &plugins),
        |path| {
            if logins == 0 {
                std::fs::write(path, login).unwrap();
            }
            logins += 1;
        },
    );
    assert_eq!(changed, Ok(true));
    assert_eq!(logins, 2);
    let text = std::fs::read_to_string(&config).unwrap();
    assert!(
        text.contains("ghcr.io") && text.contains("cli-plugins"),
        "{text}"
    );
    std::fs::remove_dir_all(&dir).ok();
}
