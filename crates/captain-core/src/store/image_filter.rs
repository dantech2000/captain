use crate::model::Image;

/// Which images the list shows.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ImageFilter {
    #[default]
    All,
    /// At least one container uses the image.
    InUse,
    /// No container uses the image.
    Unused,
    /// The image has no tag.
    Dangling,
}

impl ImageFilter {
    pub const ALL: [ImageFilter; 4] = [Self::All, Self::InUse, Self::Unused, Self::Dangling];

    pub fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::InUse => "In use",
            Self::Unused => "Unused",
            Self::Dangling => "Dangling",
        }
    }

    pub fn matches(self, image: &Image) -> bool {
        match self {
            Self::All => true,
            Self::InUse => image.in_use(),
            Self::Unused => !image.in_use(),
            Self::Dangling => image.dangling,
        }
    }
}

#[cfg(test)]
mod tests;
