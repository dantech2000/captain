use super::{ContainerState, Image, Volume};

/// What the engine stores on its disk, like `docker system df -v`. The Storage page
/// reads it. See docs/features/0031-storage.md.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DiskUsage {
    /// Tagged and dangling images. Intermediate build layers are not listed.
    pub images: Vec<Image>,
    pub containers: Vec<DiskContainer>,
    pub volumes: Vec<Volume>,
    pub build_cache: Vec<BuildCacheRecord>,
    /// The bytes of all image layers. Layers that images share count once, so this
    /// is less than the sum of the image sizes.
    pub images_bytes: u64,
    /// The bytes of all writable container layers.
    pub containers_bytes: u64,
    pub volumes_bytes: u64,
    pub build_cache_bytes: u64,
}

impl DiskUsage {
    /// Images, containers, volumes, and build cache together.
    pub fn total_bytes(&self) -> u64 {
        self.images_bytes + self.containers_bytes + self.volumes_bytes + self.build_cache_bytes
    }
}

/// A container and what it takes on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskContainer {
    pub id: String,
    /// The name without the leading `/`.
    pub name: String,
    /// The full ID of its image, for example `sha256:4a3f…`.
    pub image_id: String,
    pub state: ContainerState,
    /// The human status from the engine, for example "Exited (0) 5 days ago".
    pub status: String,
    /// Creation time as a Unix timestamp in seconds.
    pub created: i64,
    /// The bytes of its writable layer.
    pub size_rw: u64,
    /// The names of the volumes it mounts.
    pub volumes: Vec<String>,
}

/// One record of the BuildKit build cache.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BuildCacheRecord {
    pub id: String,
    /// The build step that made it, for example `[2/5] RUN npm ci`.
    pub description: String,
    /// The record type, for example `regular`, `exec.cachemount`, or `internal`.
    pub kind: String,
    pub size: u64,
    /// True while a build uses it.
    pub in_use: bool,
    /// True if other records share its data, so removing it may free less.
    pub shared: bool,
    /// Unix seconds. Zero if unknown.
    pub created: i64,
    /// Unix seconds. `None` if it was never used after it was made.
    pub last_used: Option<i64>,
}

impl BuildCacheRecord {
    /// When a build last used the record, or when it was made. The build prune's
    /// `until` filter compares this time.
    pub fn last_active(&self) -> i64 {
        self.last_used.unwrap_or(self.created)
    }

    /// True for the internal and frontend records that a build prune keeps unless
    /// it runs with `all`.
    pub fn is_internal(&self) -> bool {
        matches!(self.kind.as_str(), "internal" | "frontend")
    }
}
