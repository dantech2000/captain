use serde_json::{Map, Value, json};

use super::{ACTIVITY_CAP, Activity, append_activity, read_activity};

fn arguments(value: Value) -> Map<String, Value> {
    value.as_object().cloned().unwrap_or_default()
}

#[test]
fn reads_the_newest_entries_first_and_skips_broken_lines() {
    let dir = std::env::temp_dir().join(format!("captain-activity-{}", std::process::id()));
    let path = dir.join("agent-activity.jsonl");
    std::fs::remove_dir_all(&dir).ok();
    let first = Activity::new(
        1,
        "claude-code",
        "list_projects",
        Map::new(),
        true,
        "2 projects",
    );
    let second = Activity::new(
        2,
        "codex",
        "restart",
        arguments(json!({ "container": "shop-api-1" })),
        false,
        "The restart action is off.\nmore",
    );
    append_activity(&path, &first).unwrap();
    std::fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .and_then(|mut file| std::io::Write::write_all(&mut file, b"not json\n"))
        .unwrap();
    append_activity(&path, &second).unwrap();
    let read = read_activity(&path, 10);
    assert_eq!(read, [second.clone(), first]);
    assert_eq!(read[0].summary(), "restart shop-api-1");
    assert_eq!(read[0].result, "The restart action is off.");
    assert!(read[0].is_action());
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn the_file_stays_under_its_cap() {
    let dir = std::env::temp_dir().join(format!("captain-activity-cap-{}", std::process::id()));
    let path = dir.join("agent-activity.jsonl");
    std::fs::remove_dir_all(&dir).ok();
    let long = "x".repeat(1000);
    for at in 0..600 {
        let entry = Activity::new(
            at,
            "zed",
            "logs",
            arguments(json!({ "grep": long })),
            true,
            &long,
        );
        append_activity(&path, &entry).unwrap();
    }
    assert!(std::fs::metadata(&path).unwrap().len() <= ACTIVITY_CAP);
    let read = read_activity(&path, 1000);
    assert_eq!(read[0].at, 599);
    assert!(read.windows(2).all(|pair| pair[0].at == pair[1].at + 1));
    std::fs::remove_dir_all(&dir).ok();
}
