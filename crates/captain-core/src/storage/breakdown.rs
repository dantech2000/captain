use crate::model::DiskUsage;

/// One part of the disk bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Images,
    Volumes,
    BuildCache,
    Snapshots,
    /// The writable layers of containers.
    Containers,
}

impl Category {
    pub fn label(self) -> &'static str {
        match self {
            Self::Images => "Images",
            Self::Volumes => "Volumes",
            Self::BuildCache => "Build cache",
            Self::Snapshots => "Snapshots",
            Self::Containers => "Containers",
        }
    }
}

/// A category's bytes and its part of the bar, from 0 to 1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CategoryShare {
    pub category: Category,
    pub bytes: u64,
    pub fraction: f32,
}

/// The header of the Storage page: the bytes in use, the disk size, and the bar.
#[derive(Debug, Clone, PartialEq)]
pub struct DiskBreakdown {
    pub used: u64,
    /// The size of Captain Engine's disk. `None` for an engine Captain does not run.
    pub capacity: Option<u64>,
    pub categories: Vec<CategoryShare>,
}

impl DiskBreakdown {
    /// The categories of `usage`, in bar order. `snapshots` is the bytes of Captain
    /// Engine's snapshots, `None` for other engines, which then have no Snapshots
    /// part. Each fraction is of `capacity`, or of the bytes in use without one.
    pub fn new(usage: &DiskUsage, snapshots: Option<u64>, capacity: Option<u64>) -> Self {
        let mut parts = vec![
            (Category::Images, usage.images_bytes),
            (Category::Volumes, usage.volumes_bytes),
            (Category::BuildCache, usage.build_cache_bytes),
        ];
        parts.extend(snapshots.map(|bytes| (Category::Snapshots, bytes)));
        parts.push((Category::Containers, usage.containers_bytes));
        let used: u64 = parts.iter().map(|(_, bytes)| bytes).sum();
        let whole = capacity.unwrap_or(used).max(used).max(1) as f64;
        let categories = parts
            .into_iter()
            .map(|(category, bytes)| CategoryShare {
                category,
                bytes,
                fraction: (bytes as f64 / whole) as f32,
            })
            .collect();
        Self {
            used,
            capacity,
            categories,
        }
    }
}

#[cfg(test)]
mod tests;
