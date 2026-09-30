use std::path::Path;

use super::{REGENERATE, markdown};

#[test]
fn the_committed_reference_matches_the_tools() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/reference/mcp.md");
    let committed = std::fs::read_to_string(&path)
        .unwrap_or_default()
        .replace("\r\n", "\n");
    assert!(
        committed == markdown(),
        "docs/reference/mcp.md is out of date. Run `{REGENERATE}`."
    );
}
