use std::collections::HashMap;

use super::memory_used;

#[test]
fn memory_subtracts_inactive_files_like_docker_stats() {
    let stats = |pairs: &[(&str, u64)]| -> HashMap<String, u64> {
        pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    };
    // cgroup v1 with child cgroups: the total, not the local value.
    let v1 = stats(&[
        ("inactive_file", 10),
        ("total_inactive_file", 100),
        ("cache", 50),
    ]);
    assert_eq!(memory_used(200, &v1), 100);
    assert_eq!(memory_used(200, &stats(&[("inactive_file", 30)])), 170);
    // A value not below the usage leaves the usage as it is.
    assert_eq!(memory_used(200, &stats(&[("inactive_file", 300)])), 200);
}
