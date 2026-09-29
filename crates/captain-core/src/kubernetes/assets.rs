//! The k3s release files Captain downloads: the same names Rancher Desktop uses
//! (k3sHelper.ts). See ADR 0010.

/// The three files of one k3s release for one CPU.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct K3sAssets {
    /// `k3s` or `k3s-arm64`.
    pub binary: &'static str,
    /// The system images, so k3s starts without pulling them.
    pub images: &'static str,
    /// `sha256sum` output for the release files.
    pub checksums: &'static str,
}

impl K3sAssets {
    /// The files for `arch` as Rust names it (`std::env::consts::ARCH`). The guest
    /// has the Mac's CPU.
    pub fn for_arch(arch: &str) -> Option<Self> {
        match arch {
            "aarch64" => Some(Self {
                binary: "k3s-arm64",
                images: "k3s-airgap-images-arm64.tar.zst",
                checksums: "sha256sum-arm64.txt",
            }),
            "x86_64" => Some(Self {
                binary: "k3s",
                images: "k3s-airgap-images-amd64.tar.zst",
                checksums: "sha256sum-amd64.txt",
            }),
            _ => None,
        }
    }
}

/// `https://github.com/k3s-io/k3s/releases/download/v1.36.4%2Bk3s1/<file>`.
pub fn download_url(version: &str, file: &str) -> String {
    format!(
        "https://github.com/k3s-io/k3s/releases/download/{}/{file}",
        version.replace('+', "%2B")
    )
}

/// The SHA-256 of `file` in a `sha256sum` listing, in lower case.
pub fn expected_sha256(listing: &str, file: &str) -> Option<String> {
    listing.lines().find_map(|line| {
        let (hash, name) = line.split_once(char::is_whitespace)?;
        let name = name.trim_start().trim_start_matches('*');
        (name == file && hash.len() == 64).then(|| hash.to_ascii_lowercase())
    })
}

#[cfg(test)]
mod tests;
