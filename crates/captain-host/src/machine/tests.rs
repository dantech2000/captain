use captain_core::GIB;

use super::{parse_meminfo, parse_sysctl};

#[test]
fn reads_sysctl_and_meminfo() {
    assert_eq!(parse_sysctl("17179869184\n"), Some(16 * GIB));
    assert_eq!(parse_sysctl("0"), None);
    assert_eq!(parse_sysctl(""), None);
    let meminfo = "MemTotal:       16384000 kB\nMemFree:         1000 kB\n";
    assert_eq!(parse_meminfo(meminfo), Some(16_384_000 * 1024));
    assert_eq!(parse_meminfo("MemFree: 1 kB\n"), None);
}
