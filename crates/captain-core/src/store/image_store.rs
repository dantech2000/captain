use std::cmp::Ordering;

use super::ImageFilter;
use crate::model::Image;

/// The image list that views render. It keeps images in display order: tagged images
/// by name, then untagged images, newest first.
#[derive(Debug, Clone, Default)]
pub struct ImageStore {
    images: Vec<Image>,
}

impl ImageStore {
    /// Replaces the whole list with a fresh one from the engine.
    pub fn replace(&mut self, mut images: Vec<Image>) {
        images.sort_by(display_order);
        self.images = images;
    }

    pub fn images(&self) -> &[Image] {
        &self.images
    }

    pub fn len(&self) -> usize {
        self.images.len()
    }

    pub fn is_empty(&self) -> bool {
        self.images.is_empty()
    }

    pub fn find(&self, id: &str) -> Option<&Image> {
        self.images.iter().find(|image| image.id == id)
    }

    /// Images that pass `filter`, in display order.
    pub fn filtered(&self, filter: ImageFilter) -> Vec<&Image> {
        self.images.iter().filter(|i| filter.matches(i)).collect()
    }

    /// The number of images that pass `filter`.
    pub fn count(&self, filter: ImageFilter) -> usize {
        self.images.iter().filter(|i| filter.matches(i)).count()
    }

    /// The size of every image, in bytes. Shared layers count once per image, as
    /// `docker images` shows them, so this can be more than the disk use.
    pub fn total_size(&self) -> u64 {
        self.images.iter().map(|i| i.size).sum()
    }

    /// The bytes that pruning dangling images would free, at most.
    pub fn dangling_size(&self) -> u64 {
        self.images
            .iter()
            .filter(|i| i.dangling && !i.in_use())
            .map(|i| i.size)
            .sum()
    }
}

fn display_order(a: &Image, b: &Image) -> Ordering {
    match (a.repo_tags.first(), b.repo_tags.first()) {
        (Some(a_tag), Some(b_tag)) => a_tag.cmp(b_tag),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => b.created.cmp(&a.created),
    }
}

#[cfg(test)]
mod tests;
