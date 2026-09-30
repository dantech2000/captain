use std::collections::HashMap;
use std::time::{Duration, Instant};

use captain_core::model::ContainerDetail;

use super::ExitFactsCache;

fn killed(started_at: &str) -> ContainerDetail {
    ContainerDetail {
        id: "web".into(),
        started_at: started_at.into(),
        oom_killed: true,
        memory_limit: 256 << 20,
        restart_count: 3,
        ..Default::default()
    }
}

#[test]
fn inspects_each_new_crash_once_and_forgets_recovered_containers() {
    let mut cache = ExitFactsCache::default();
    let first = Instant::now();
    let crashing = |at: Instant| HashMap::from([("web".to_string(), Some(at))]);

    assert_eq!(cache.follow(crashing(first)), ["web"]);
    assert!(cache.insert("web".into(), killed("14:00")));
    assert!(cache.follow(crashing(first)).is_empty());

    // A new crash asks again, and keeps the old facts until the answer comes.
    assert_eq!(
        cache.follow(crashing(first + Duration::from_secs(5))),
        ["web"]
    );
    assert!(cache.facts()["web"].oom_killed);

    assert!(cache.follow(HashMap::new()).is_empty());
    assert!(cache.facts().is_empty());
    assert!(!cache.insert("web".into(), killed("14:00")));
}

#[test]
fn a_raise_waits_for_a_run_with_the_new_limit() {
    let mut cache = ExitFactsCache::default();
    cache.follow(HashMap::from([("web".to_string(), None)]));
    cache.insert("web".into(), killed("14:00"));

    cache.record_raise("web");
    assert!(cache.facts()["web"].raised);

    assert!(cache.insert("web".into(), killed("14:05")));
    assert!(!cache.facts()["web"].raised);
}
