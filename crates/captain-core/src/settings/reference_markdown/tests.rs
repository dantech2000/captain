use std::path::PathBuf;

use super::reference_markdown;
use crate::settings::settings_schema_text;

/// docs/reference/settings.md and settings.schema.json match the settings types.
/// `CAPTAIN_BLESS=1` writes them instead.
#[test]
fn the_committed_reference_matches_the_settings_types() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/reference");
    for (name, generated) in [
        ("settings.md", reference_markdown()),
        ("settings.schema.json", settings_schema_text()),
    ] {
        let path = dir.join(name);
        if std::env::var_os("CAPTAIN_BLESS").is_some() {
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(&path, &generated).unwrap();
            continue;
        }
        // Git may check the files out with CRLF line ends on Windows.
        let committed = std::fs::read_to_string(&path)
            .unwrap_or_default()
            .replace("\r\n", "\n");
        assert!(
            committed == generated,
            "docs/reference/{name} is out of date. Run \
             `CAPTAIN_BLESS=1 cargo test -p captain-core reference` and commit it."
        );
    }
}
