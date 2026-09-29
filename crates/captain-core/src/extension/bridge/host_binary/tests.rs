use std::path::Path;

use super::host_binary;

#[test]
fn only_bundled_binaries_run() {
    let bin = Path::new("/ext/bin");
    let installed = ["tool".to_string(), "helper.exe".to_string()];
    assert_eq!(host_binary(bin, &installed, "tool"), Ok(bin.join("tool")));
    assert_eq!(
        host_binary(bin, &installed, "helper"),
        Ok(bin.join("helper.exe"))
    );
    assert!(host_binary(bin, &installed, "sh").is_err());
    assert!(host_binary(bin, &installed, "/bin/tool").is_err());
    assert!(host_binary(bin, &installed, "../bin/tool").is_err());
}
