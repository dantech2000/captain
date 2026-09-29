/// A group of items. Steps run in this order, because later ones need earlier ones:
/// containers attach to networks and mount volumes, and projects need their images.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Step {
    Networks,
    Volumes,
    Images,
    ComposeProjects,
    Containers,
}

impl Step {
    pub const ALL: [Step; 5] = [
        Step::Networks,
        Step::Volumes,
        Step::Images,
        Step::ComposeProjects,
        Step::Containers,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Step::Networks => "Networks",
            Step::Volumes => "Volumes",
            Step::Images => "Images",
            Step::ComposeProjects => "Compose projects",
            Step::Containers => "Containers",
        }
    }
}
