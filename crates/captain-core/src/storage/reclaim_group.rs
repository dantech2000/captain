use std::time::Duration;

const DAY: u64 = 24 * 60 * 60;

/// Build cache that no build used for this long is old.
pub const BUILD_CACHE_AGE: Duration = Duration::from_secs(14 * DAY);
/// Stopped containers created this long ago are old.
pub const STOPPED_AGE: Duration = Duration::from_secs(3 * DAY);

/// One checkbox of the cleanup panel. Nothing a container uses is in any group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ReclaimGroup {
    OldBuildCache,
    DanglingImages,
    UnusedImages,
    OldStoppedContainers,
    UnusedVolumes,
}

impl ReclaimGroup {
    /// The groups in the order the panel lists them.
    pub const ALL: [Self; 5] = [
        Self::OldBuildCache,
        Self::DanglingImages,
        Self::UnusedImages,
        Self::OldStoppedContainers,
        Self::UnusedVolumes,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::OldBuildCache => "Build cache older than 14 days",
            Self::DanglingImages => "Dangling images",
            Self::UnusedImages => "Images no container uses",
            Self::OldStoppedContainers => "Stopped containers older than 3 days",
            Self::UnusedVolumes => "Volumes no container uses",
        }
    }

    /// Every group but volumes starts checked. Volumes hold data that cannot come
    /// back without a snapshot.
    pub fn checked_by_default(self) -> bool {
        !self.has_data()
    }

    /// True if removing the group's items loses data, not only something that a
    /// pull or a build makes again.
    pub fn has_data(self) -> bool {
        self == Self::UnusedVolumes
    }
}
