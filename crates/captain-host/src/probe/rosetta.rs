/// Where macOS keeps Rosetta for Linux VMs. Lima's `vz` driver shares this folder
/// with the VM (`VZLinuxRosettaDirectoryShare`), so Captain Engine needs it, not
/// the Rosetta that runs Mac apps.
const ROSETTA_LINUX: &str = "/Library/Apple/usr/libexec/oah/RosettaLinux";

/// Whether Rosetta for Linux is installed. `None` where it does not apply:
/// anything but an Apple silicon Mac.
pub fn rosetta_installed() -> Option<bool> {
    if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        Some(std::path::Path::new(ROSETTA_LINUX).exists())
    } else {
        None
    }
}
