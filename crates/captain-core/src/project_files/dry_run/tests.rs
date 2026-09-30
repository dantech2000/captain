//! Real stderr of Compose v5.5.1 against Captain Engine.

use super::*;

/// `web` got a new image and `cache` is a new service.
const RECREATE: &str = r#"{"id":"Container captain-agent-m32fix-api-1","status":"Done","text":"Running"}
{"id":"Container captain-agent-m32fix-web-1","status":"Working","text":"Recreate"}
{"id":"Container captain-agent-m32fix-cache-1","status":"Working","text":"Creating"}
{"id":"Container captain-agent-m32fix-cache-1","status":"Done","text":"Created"}
{"id":"Container captain-agent-m32fix-web-1","status":"Done","text":"Recreated"}
{"id":"Container captain-agent-m32fix-cache-1","status":"Working","text":"Starting"}
{"id":"Container 00fccd82ddf9_captain-agent-m32fix-web-1","status":"Working","text":"Starting"}
"#;

/// `web` left the file (`--remove-orphans`), `cache` pulls, `api` builds.
const REMOVE: &str = r#"{"id":"Image redis:7-alpine","status":"Working","text":"Pulling"}
{"id":"Image redis:7-alpine","status":"Done","text":"Pulled"}
{"id":"Image captain-agent-m32fix-api","status":"Working","text":"Building"}
{"id":"api ==>","status":"Done","text":"==> writing image dryRun-a033a528b603"}
{"id":"Image api","status":"Done","text":"Built"}
{"id":"Container captain-agent-m32fix-api-1","status":"Done","text":"Running"}
{"id":"Container captain-agent-m32fix-web-1","status":"Working","text":"Stopping"}
{"id":"Container captain-agent-m32fix-cache-1","status":"Working","text":"Creating"}
{"level":"warning","msg":"Found orphan containers (captain-agent-m32fix-web-1) for this project.","time":"2026-09-30T13:11:35-07:00"}
"#;

fn known() -> Vec<(String, String)> {
    vec![
        ("captain-agent-m32fix-api-1".into(), "api".into()),
        ("captain-agent-m32fix-web-1".into(), "web".into()),
    ]
}

fn summary(preview: &UpPreview) -> Vec<(&str, &str)> {
    preview
        .changes
        .iter()
        .map(|c| (c.service.as_str(), c.change.label()))
        .collect()
}

#[test]
fn reads_the_first_action_of_each_container_and_skips_fake_copies() {
    let preview = parse_dry_run(RECREATE, "captain-agent-m32fix", &known());
    assert_eq!(
        summary(&preview),
        [
            ("api", "No change"),
            ("cache", "Create"),
            ("web", "Recreate")
        ]
    );
    assert!(preview.images.is_empty());
    assert!(!preview.is_empty());
}

#[test]
fn reads_pulls_builds_removals_and_warnings() {
    let preview = parse_dry_run(REMOVE, "captain-agent-m32fix", &known());
    assert_eq!(
        summary(&preview),
        [("api", "No change"), ("cache", "Create"), ("web", "Remove")]
    );
    assert_eq!(
        preview.images,
        [
            ImageStep {
                name: "redis:7-alpine".into(),
                build: false
            },
            ImageStep {
                name: "api".into(),
                build: true
            }
        ]
    );
    assert_eq!(preview.warnings.len(), 1);
}

#[test]
fn keeps_a_real_container_whose_name_looks_like_a_fake_copy() {
    let stderr = r#"{"id":"Container 0123456789ab_api","status":"Working","text":"Recreate"}"#;
    let known = [("0123456789ab_api".to_string(), "api".to_string())];
    let preview = parse_dry_run(stderr, "shop", &known);
    assert_eq!(summary(&preview), [("api", "Recreate")]);
}
