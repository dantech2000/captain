use std::path::Path;

use super::{add_plugin_dir, with_plugin_dir, without_plugin_dir};

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

#[cfg(unix)]
#[test]
fn writes_through_a_link_and_backs_up_once() {
    let dir = std::env::temp_dir().join(format!("captain-plugin-config-{}", std::process::id()));
    std::fs::remove_dir_all(&dir).ok();
    std::fs::create_dir_all(dir.join("dotfiles")).unwrap();
    let target = dir.join("dotfiles/config.json");
    std::fs::write(&target, r#"{"credsStore":"osxkeychain"}"#).unwrap();
    let link = dir.join("config.json");
    std::os::unix::fs::symlink(&target, &link).unwrap();

    assert_eq!(add_plugin_dir(&link, Path::new(DIR)), Ok(true));
    assert!(
        std::fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert!(std::fs::read_to_string(&target).unwrap().contains(DIR));
    let backup = dir.join("config.json.captain-backup");
    assert_eq!(
        std::fs::read_to_string(&backup).unwrap(),
        r#"{"credsStore":"osxkeychain"}"#
    );

    assert_eq!(add_plugin_dir(&link, Path::new("/other")), Ok(true));
    assert_eq!(
        std::fs::read_to_string(&backup).unwrap(),
        r#"{"credsStore":"osxkeychain"}"#
    );
    std::fs::remove_dir_all(&dir).ok();
}
