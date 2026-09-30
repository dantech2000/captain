//! What the Settings page shows about Captain Engine's files: the folder, the
//! runtime version, and the free disk. See docs/features/0037-settings-page.md.

use std::path::PathBuf;

use gpui_kit::*;

use super::HostModel;

impl HostModel {
    /// The folder with the engine's files, for Show engine files.
    pub fn files_dir(&self) -> Option<PathBuf> {
        self.host.files_dir()
    }

    /// What runs the engine, such as `Lima 2.0.3`.
    pub fn runtime_version(&self) -> Option<String> {
        self.host.runtime_version()
    }

    /// Reads the free disk in the background.
    pub fn free_disk(&self, cx: &App) -> Task<Option<u64>> {
        let host = self.host.clone();
        cx.background_executor()
            .spawn(async move { host.free_disk() })
    }
}
