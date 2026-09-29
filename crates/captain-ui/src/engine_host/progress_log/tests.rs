use super::ProgressLog;

#[test]
fn keeps_the_latest_lines() {
    let mut log = ProgressLog::default();
    for n in 0..250 {
        log.push(format!("line {n}"));
    }
    let last: Vec<&str> = log.last(3).collect();
    assert_eq!(last, ["line 247", "line 248", "line 249"]);
    assert_eq!(log.last(1000).count(), 200);
}

#[test]
fn clears() {
    let mut log = ProgressLog::default();
    log.push("one".into());
    assert!(!log.is_empty());
    log.clear();
    assert!(log.is_empty());
    assert_eq!(log.last(5).count(), 0);
}
