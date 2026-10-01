use super::{GIB, HostResources};

#[test]
fn half_the_cpus_between_two_and_eight() {
    assert_eq!(HostResources::recommended(2, 8 * GIB).cpus, 2);
    assert_eq!(HostResources::recommended(10, 8 * GIB).cpus, 5);
    assert_eq!(HostResources::recommended(24, 8 * GIB).cpus, 8);
    assert_eq!(HostResources::recommended(1, 8 * GIB).cpus, 2);
}

#[test]
fn memory_is_a_quarter_between_four_and_sixteen_gib() {
    assert_eq!(HostResources::recommended(8, 8 * GIB).memory_bytes, 4 * GIB);
    assert_eq!(
        HostResources::recommended(8, 32 * GIB).memory_bytes,
        8 * GIB
    );
    assert_eq!(
        HostResources::recommended(8, 128 * GIB).memory_bytes,
        16 * GIB
    );
}

#[test]
fn disk_is_sixty_four_gib() {
    assert_eq!(HostResources::recommended(8, 16 * GIB).disk_bytes, 64 * GIB);
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

#[test]
fn summary_names_each_resource() {
    let resources = HostResources {
        cpus: 4,
        memory_bytes: 8 * GIB,
        disk_bytes: 64 * GIB,
    };
    assert_eq!(resources.summary(), "4 CPUs · 8.0 GB memory · 64.0 GB disk");
}
