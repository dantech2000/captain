use std::collections::HashMap;

use super::ComposeWalk;

#[test]
fn included_files_name_more_files_relative_to_their_folder_and_a_cycle_ends() {
    let files = HashMap::from([
        ("compose.yaml", "include: [child/compose.yaml]"),
        (
            "child/compose.yaml",
            "include: [grand.yaml, ../escape.yaml]\nservices: {a: {env_file: a.env}}",
        ),
        ("child/grand.yaml", "include: [grand.yaml]"),
    ]);
    let mut walk = ComposeWalk::new("compose.yaml");
    let mut copied = Vec::new();
    while let Some((file, depth)) = walk.next_file() {
        copied.extend(walk.read(&file, depth, files[file.as_str()]));
    }
    copied.sort();
    assert_eq!(
        copied,
        [
            ".env",
            "child/.env",
            "child/a.env",
            "child/compose.yaml",
            "child/grand.yaml"
        ]
    );
}
