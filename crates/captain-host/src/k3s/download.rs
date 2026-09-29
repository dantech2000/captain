//! Downloads one k3s version into `~/.captain/cache/k3s/<version>/`: the binary, the
//! air-gap images, and the checksums. The files land in a temporary folder, and the
//! folder is renamed only after each SHA-256 matches.

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use captain_core::HostError;
use captain_core::kubernetes::{K3sAssets, K3sVersion, download_url, expected_sha256};
use sha2::{Digest, Sha256};

use super::curl;

/// The folder that holds `version` in `cache`.
pub fn folder(cache: &Path, version: &K3sVersion) -> PathBuf {
    cache.join(version.as_str())
}

/// The version's folder, after a download if it is missing.
pub fn ensure(
    cache: &Path,
    version: &K3sVersion,
    assets: &K3sAssets,
    sink: &mut dyn FnMut(String),
) -> Result<PathBuf, HostError> {
    let done = folder(cache, version);
    if done.join(assets.binary).is_file() && done.join(assets.images).is_file() {
        return Ok(done);
    }
    let temp = cache.join(format!(".{version}.partial"));
    std::fs::remove_dir_all(&temp).ok();
    let io = |error: std::io::Error| HostError(format!("Cannot write {}: {error}", temp.display()));
    std::fs::create_dir_all(&temp).map_err(io)?;
    let result = fetch(&temp, version, assets, sink).and_then(|()| {
        std::fs::remove_dir_all(&done).ok();
        std::fs::rename(&temp, &done).map_err(io)
    });
    if result.is_err() {
        std::fs::remove_dir_all(&temp).ok();
    }
    result.map(|()| done)
}

fn fetch(
    temp: &Path,
    version: &K3sVersion,
    assets: &K3sAssets,
    sink: &mut dyn FnMut(String),
) -> Result<(), HostError> {
    let listing = curl::text(&download_url(version.as_str(), assets.checksums))?;
    for (file, what) in [(assets.binary, "k3s"), (assets.images, "the system images")] {
        sink(format!("Downloading {what} for Kubernetes {version}."));
        let path = temp.join(file);
        curl::save(&download_url(version.as_str(), file), &path)?;
        let expected = expected_sha256(&listing, file).ok_or_else(|| {
            HostError(format!("{} has no checksum for {file}.", assets.checksums))
        })?;
        let actual = sha256(&path)?;
        if actual != expected {
            return Err(HostError(format!(
                "The download of {file} is damaged: its SHA-256 is {actual}, not {expected}."
            )));
        }
    }
    std::fs::write(temp.join(assets.checksums), listing)
        .map_err(|error| HostError(error.to_string()))
}

fn sha256(path: &Path) -> Result<String, HostError> {
    let io = |error: std::io::Error| HostError(format!("Cannot read {}: {error}", path.display()));
    let mut file = File::open(path).map_err(io)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0; 1 << 20];
    loop {
        let read = file.read(&mut buffer).map_err(io)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg(test)]
mod tests;
