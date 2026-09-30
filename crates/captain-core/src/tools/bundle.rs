//! Where the bundled tools live, and the order Captain looks for a tool in.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// The tools inside `Captain.app/Contents/Resources`. The layout matches what
/// `scripts/bundle-macos.sh` copies from `scripts/fetch-tools.sh`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bundle {
    resources: PathBuf,
}

impl Bundle {
    /// The bundle for an executable at `exe`, when it runs from an app bundle: the
    /// app (`Captain.app/Contents/MacOS/captain`) or the CLI
    /// (`Captain.app/Contents/Resources/bin/captain`). `None` for a `cargo run` build,
    /// and on Linux and Windows, whose packages do not ship tools yet.
    pub fn from_exe(exe: &Path) -> Option<Self> {
        let dir = exe.parent()?;
        let contents = if dir.file_name()? == "MacOS" {
            dir.parent()?
        } else if dir.file_name()? == "bin" {
            let resources = dir.parent()?;
            (resources.file_name()? == "Resources").then_some(resources.parent()?)?
        } else {
            return None;
        };
        (contents.file_name()? == "Contents").then(|| Self {
            resources: contents.join("Resources"),
        })
    }

    /// `lima/bin/limactl`. Lima finds its templates in `lima/share/lima` next to it.
    pub fn limactl(&self) -> PathBuf {
        self.resources.join("lima/bin/limactl")
    }

    /// `bin`, with `docker`, `docker-credential-osxkeychain`, `kubectl`, `helm`, and
    /// `captain`.
    pub fn bin(&self) -> PathBuf {
        self.resources.join("bin")
    }

    /// `bin/docker`.
    pub fn docker(&self) -> PathBuf {
        self.resources.join("bin/docker")
    }

    /// `bin/docker-credential-osxkeychain`, the macOS credential helper.
    pub fn credential_helper(&self) -> PathBuf {
        self.resources.join("bin/docker-credential-osxkeychain")
    }

    /// `bin/captain`, the command line.
    pub fn captain_cli(&self) -> PathBuf {
        self.resources.join("bin/captain")
    }

    /// `licenses`, the license texts of the bundled tools.
    pub fn licenses(&self) -> PathBuf {
        self.resources.join("licenses")
    }

    /// `cli-plugins`, with `docker-compose` and `docker-buildx`.
    pub fn cli_plugins(&self) -> PathBuf {
        self.resources.join("cli-plugins")
    }
}

/// The first `binary` that exists: the `bundled` copy, then each folder in `path`
/// (a `PATH`-style list), then each of `fallback_dirs`. An app started from the
/// Finder or a desktop launcher gets a short `PATH`, so known install folders come
/// last. `exists` checks whether a file exists.
pub fn locate_tool(
    binary: &str,
    bundled: Option<PathBuf>,
    path: Option<&OsStr>,
    fallback_dirs: impl IntoIterator<Item = PathBuf>,
    exists: impl Fn(&Path) -> bool,
) -> Option<PathBuf> {
    let dirs = path
        .into_iter()
        .flat_map(std::env::split_paths)
        .chain(fallback_dirs);
    bundled
        .into_iter()
        .chain(dirs.map(|dir| dir.join(binary)))
        .find(|candidate| exists(candidate))
}

#[cfg(test)]
mod tests;
