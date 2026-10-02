use super::{GIB, HostResources};

#[test]
fn the_recommendation_is_half_the_cpus_a_quarter_of_memory_and_64_gib_of_disk() {
    let cpus = [(2, 2), (10, 5), (24, 8), (1, 2)];
    for (machine, recommended) in cpus {
        let resources = HostResources::recommended(machine, 8 * GIB);
        assert_eq!(resources.cpus, recommended, "{machine} CPUs");
    }
    let memory = [(8, 4), (32, 8), (128, 16)];
    for (machine, recommended) in memory {
        let resources = HostResources::recommended(8, machine * GIB);
        assert_eq!(resources.memory_bytes, recommended * GIB, "{machine} GiB");
    }
    let resources = HostResources::recommended(8, 16 * GIB);
    assert_eq!(resources.disk_bytes, 64 * GIB);
}

#[test]
fn steps_stay_in_range() {
    let base = HostResources::recommended(8, 16 * GIB);
    assert_eq!(base.step_cpus(1, 8).cpus, 5);
    assert_eq!(base.step_cpus(10, 8).cpus, 8);
    assert_eq!(base.step_cpus(-10, 8).cpus, 1);

    assert_eq!(base.step_memory(1, 16 * GIB).memory_bytes, 5 * GIB);
    assert_eq!(base.step_memory(100, 16 * GIB).memory_bytes, 12 * GIB);
    assert_eq!(base.step_memory(-100, 16 * GIB).memory_bytes, 2 * GIB);

    assert_eq!(base.step_disk(-100, None).disk_bytes, 16 * GIB);
    assert_eq!(base.step_disk(16, None).disk_bytes, 80 * GIB);
}

#[test]
fn the_disk_never_steps_below_the_machine_disk() {
    let base = HostResources::recommended(8, 16 * GIB);
    let grown = base.step_disk(32, Some(64 * GIB));
    assert_eq!(grown.disk_bytes, 96 * GIB);
    assert_eq!(grown.step_disk(-16, Some(64 * GIB)).disk_bytes, 80 * GIB);
    assert_eq!(grown.step_disk(-100, Some(64 * GIB)).disk_bytes, 64 * GIB);
}

#[test]
fn a_restart_changes_only_what_differs_from_the_running_machine() {
    let running = HostResources {
        cpus: 4,
        memory_bytes: 6 * GIB,
        disk_bytes: 64 * GIB,
    };
    assert_eq!(running.restart_change(&running), None);
    // Lima keeps a disk that would shrink, so a smaller saved disk needs no restart.
    let smaller_disk = HostResources {
        disk_bytes: 32 * GIB,
        ..running
    };
    assert_eq!(smaller_disk.restart_change(&running), None);
    let saved = HostResources {
        cpus: 6,
        memory_bytes: 8 * GIB,
        ..running
    };
    assert_eq!(
        saved.restart_change(&running),
        Some((
            "4 CPUs and 6.0 GB memory".into(),
            "6 CPUs and 8.0 GB memory".into()
        ))
    );
}
