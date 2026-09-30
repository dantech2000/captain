use std::collections::HashMap;
use std::time::{Duration, Instant};

use captain_core::problems::ExitFacts;

use super::ExitFactsCache;

#[test]
fn inspects_each_new_crash_once_and_forgets_recovered_containers() {
    let mut cache = ExitFactsCache::default();
    let first = Instant::now();
    let crashing = |at: Instant| HashMap::from([("web".to_string(), Some(at))]);

    assert_eq!(cache.follow(crashing(first)), ["web"]);
    let facts = ExitFacts {
        oom_killed: true,
        memory_limit: 256 << 20,
        restart_count: 3,
    };
    assert!(cache.insert("web".into(), facts));
    assert!(cache.follow(crashing(first)).is_empty());

    // A new crash asks again, and keeps the old facts until the answer comes.
    assert_eq!(
        cache.follow(crashing(first + Duration::from_secs(5))),
        ["web"]
    );
    assert_eq!(cache.facts().get("web"), Some(&facts));

    assert!(cache.follow(HashMap::new()).is_empty());
    assert!(cache.facts().is_empty());
    assert!(!cache.insert("web".into(), facts));
}
