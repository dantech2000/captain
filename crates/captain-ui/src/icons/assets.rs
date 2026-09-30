use std::borrow::Cow;

use gpui_kit::assets::AllAssets;
use gpui_kit::{AssetSource, Result, SharedString};

/// The folder of Captain's own assets, next to GPUI Kit's `icons/`.
const PREFIX: &str = "captain/";

/// Every file in `assets/icons/`, compiled in.
macro_rules! icon_files {
    ($($name:literal),* $(,)?) => {
        &[$((
            concat!("captain/icons/", $name),
            include_bytes!(concat!("../../../../assets/icons/", $name)) as &[u8],
        )),*]
    };
}

static FILES: &[(&str, &[u8])] = icon_files![
    "cluster-fill.svg",
    "cluster-fill2.svg",
    "cluster-line.svg",
    "container-fill.svg",
    "container-line.svg",
    "engine-fill.svg",
    "engine-fill2.svg",
    "engine-line.svg",
    "exec-fill.svg",
    "exec-line.svg",
    "extension-fill.svg",
    "extension-line.svg",
    "forward-fill.svg",
    "forward-line.svg",
    "image-fill.svg",
    "image-fill2.svg",
    "image-line.svg",
    "network-fill.svg",
    "network-line.svg",
    "pod-fill.svg",
    "pod-fill2.svg",
    "pod-line.svg",
    "reclaim-fill.svg",
    "reclaim-line.svg",
    "snapshot-fill.svg",
    "snapshot-line.svg",
    "stack-fill.svg",
    "stack-line.svg",
    "volume-fill.svg",
    "volume-fill2.svg",
    "volume-line.svg",
];

/// The app's asset source: Captain's icons first, then GPUI Kit's Lucide set.
pub struct CaptainAssets;

impl AssetSource for CaptainAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        match FILES.iter().find(|(name, _)| *name == path) {
            Some((_, bytes)) => Ok(Some(Cow::Borrowed(bytes))),
            None => AllAssets.load(path),
        }
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut names: Vec<SharedString> = FILES
            .iter()
            .filter(|(name, _)| name.starts_with(path))
            .map(|(name, _)| SharedString::from(*name))
            .collect();
        if !path.starts_with(PREFIX) {
            names.extend(AllAssets.list(path)?);
        }
        Ok(names)
    }
}

#[cfg(test)]
mod tests;
