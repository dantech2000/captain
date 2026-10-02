use std::path::Path;

use captain_core::{GIB, HostResources};

use super::{create, delete, edit, list, progress_line, start, stop, stop_docker};

fn joined(args: Vec<String>) -> String {
    args.join(" ")
}

#[test]
fn builds_each_command() {
    assert_eq!(joined(list()), "list --json");
    assert_eq!(
        joined(create(
            "captain",
            Path::new("/Users/ada/.captain/captain-engine.yaml")
        )),
        "--log-format json create --name captain --tty=false /Users/ada/.captain/captain-engine.yaml"
    );
    assert_eq!(
        joined(start("captain")),
        "--log-format json start captain --tty=false --progress --timeout 30m"
    );
    assert_eq!(
        joined(stop("captain", false)),
        "--log-format json stop captain --tty=false"
    );
    assert_eq!(
        joined(stop("captain", true)),
        "--log-format json stop captain --tty=false --force"
    );
    let docker = stop_docker("captain");
    assert_eq!(
        joined(docker[..8].to_vec()),
        "shell --workdir / captain sudo -n sh -c"
    );
    assert!(docker[8].contains("exec systemctl stop $units"));
    assert_eq!(
        joined(delete("captain")),
        "--log-format json delete captain --force --tty=false"
    );
}

fn resources(cpus: u32, memory_gib: u64, disk_gib: u64) -> HostResources {
    HostResources {
        cpus,
        memory_bytes: memory_gib * GIB,
        disk_bytes: disk_gib * GIB,
    }
}

#[test]
fn edit_only_what_changed_keeping_a_larger_disk() {
    let current = resources(4, 8, 64);
    assert_eq!(edit("captain", &current, &current), None);
    assert_eq!(
        joined(edit("captain", &current, &resources(6, 8, 64)).unwrap()),
        "--log-format json edit captain --tty=false --cpus 6"
    );
    assert_eq!(
        joined(edit("captain", &current, &resources(4, 12, 100)).unwrap()),
        "--log-format json edit captain --tty=false --memory 12 --disk 100"
    );
    // A larger disk is kept.
    assert_eq!(
        edit("captain", &resources(4, 8, 100), &resources(4, 8, 64)),
        None
    );
    let wanted = HostResources {
        memory_bytes: 4 * GIB + GIB / 2,
        ..resources(4, 8, 64)
    };
    let args = joined(edit("captain", &resources(4, 8, 64), &wanted).unwrap());
    assert!(args.ends_with("--memory 4.5"), "{args}");
}

#[test]
fn progress_shows_messages() {
    let info =
        r#"{"level":"info","msg":"Downloaded the image","time":"2026-09-29T00:50:27-07:00"}"#;
    assert_eq!(progress_line(info).as_deref(), Some("Downloaded the image"));
    let warning = r#"{"level":"warning","msg":"slow","time":"t"}"#;
    assert_eq!(progress_line(warning).as_deref(), Some("warning: slow"));
    let debug = r#"{"level":"debug","msg":"noise"}"#;
    assert_eq!(progress_line(debug), None);
    assert_eq!(
        progress_line("Downloading the image (ubuntu.img)\n").as_deref(),
        Some("Downloading the image (ubuntu.img)")
    );
    assert_eq!(progress_line("   "), None);
    let editor = r#"{"level":"info","msg":"Terminal is not available, proceeding without opening an editor"}"#;
    assert_eq!(progress_line(editor), None);
}
