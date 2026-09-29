use serde_json::json;

use super::{ListOptions, OpenDialogOptions};

#[test]
fn list_filters_come_as_a_json_string_or_an_object() {
    let options = ListOptions::parse(&json!({
        "all": true,
        "filters": "{\"dangling\": [true], \"label\": {\"a=b\": true, \"c\": false}}"
    }))
    .unwrap();
    assert!(options.all);
    assert_eq!(options.filters["dangling"], ["true"]);
    assert_eq!(options.filters["label"], ["a=b"]);
    assert!(ListOptions::parse(&json!({ "filters": "{" })).is_err());
}

#[test]
fn the_open_panel_picks_files_unless_asked_for_folders() {
    let files = OpenDialogOptions::parse(&json!({}));
    assert!(files.files && !files.directories && !files.multiple);
    let folders = OpenDialogOptions::parse(&json!({
        "properties": ["openDirectory", "multiSelections"]
    }));
    assert!(!folders.files && folders.directories && folders.multiple);
}
