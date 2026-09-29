use std::path::Path;

use super::{ExtensionPaths, mime_type, ui_file};

#[test]
fn each_extension_has_its_own_folder() {
    let paths = ExtensionPaths::in_home(Path::new("/Users/me"));
    assert_eq!(
        paths.ui_dir("acme-ext"),
        Path::new("/Users/me/.captain/extensions/acme-ext/ui")
    );
    assert_eq!(
        paths.manifest("acme-ext"),
        Path::new("/Users/me/.captain/extensions/acme-ext/extension.json")
    );
}

#[test]
fn a_url_path_maps_into_the_ui_folder_only() {
    let ui = Path::new("/ext/ui");
    assert_eq!(ui_file(ui, "/"), Some(ui.join("index.html")));
    assert_eq!(
        ui_file(ui, "/assets/my%20app.js"),
        Some(ui.join("assets/my app.js"))
    );
    assert_eq!(ui_file(ui, "/../secret"), None);
    assert_eq!(ui_file(ui, "/a/%2e%2e/%2e%2e/secret"), None);
    assert_eq!(ui_file(ui, "/bad%zz"), None);
}

#[test]
fn the_content_type_follows_the_file_extension() {
    assert_eq!(
        mime_type(Path::new("a/app.JS")),
        "text/javascript; charset=utf-8"
    );
    assert_eq!(mime_type(Path::new("icon.svg")), "image/svg+xml");
    assert_eq!(mime_type(Path::new("blob")), "application/octet-stream");
}
