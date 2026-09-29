use crate::model::{Network, Volume};

/// Something that containers can use, such as a volume or a network.
pub trait Usage {
    /// True if at least one container uses it.
    fn in_use(&self) -> bool;
    /// True if it is known that no container uses it.
    fn unused(&self) -> bool;
}

impl Usage for Volume {
    fn in_use(&self) -> bool {
        self.is_in_use()
    }

    fn unused(&self) -> bool {
        self.is_unused()
    }
}

impl Usage for Network {
    fn in_use(&self) -> bool {
        self.is_in_use()
    }

    fn unused(&self) -> bool {
        !self.is_in_use()
    }
}

/// Which volumes or networks a list shows.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum UsageFilter {
    #[default]
    All,
    InUse,
    Unused,
}

impl UsageFilter {
    pub const ALL: [UsageFilter; 3] = [Self::All, Self::InUse, Self::Unused];

    pub fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::InUse => "In use",
            Self::Unused => "Unused",
        }
    }

    /// An item with an unknown usage shows only under [`UsageFilter::All`].
    pub fn matches(self, item: &impl Usage) -> bool {
        match self {
            Self::All => true,
            Self::InUse => item.in_use(),
            Self::Unused => item.unused(),
        }
    }
}

#[cfg(test)]
mod tests;
