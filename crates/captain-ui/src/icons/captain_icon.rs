/// A Captain resource glyph. Each has a line drawing, a body fill, and for some a
/// second, lighter fill; see `assets/icons/`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CaptainIcon {
    Container,
    Image,
    Volume,
    Network,
    /// A Compose project.
    Stack,
    Pod,
    /// Kubernetes.
    Cluster,
    Snapshot,
    Extension,
    Engine,
    /// A port forward.
    Forward,
    /// A shell in a container.
    Exec,
    /// Free up space.
    Reclaim,
}

impl CaptainIcon {
    pub const ALL: [CaptainIcon; 13] = [
        CaptainIcon::Container,
        CaptainIcon::Image,
        CaptainIcon::Volume,
        CaptainIcon::Network,
        CaptainIcon::Stack,
        CaptainIcon::Pod,
        CaptainIcon::Cluster,
        CaptainIcon::Snapshot,
        CaptainIcon::Extension,
        CaptainIcon::Engine,
        CaptainIcon::Forward,
        CaptainIcon::Exec,
        CaptainIcon::Reclaim,
    ];

    /// The asset path of the stroked line drawing.
    pub fn line_path(self) -> &'static str {
        self.paths().0
    }

    /// The asset path of the body fill.
    pub fn fill_path(self) -> &'static str {
        self.paths().1
    }

    /// The asset path of the second fill, for glyphs that have one.
    pub fn second_fill_path(self) -> Option<&'static str> {
        self.paths().2
    }

    fn paths(self) -> (&'static str, &'static str, Option<&'static str>) {
        macro_rules! files {
            ($name:literal) => {
                (
                    concat!("captain/icons/", $name, "-line.svg"),
                    concat!("captain/icons/", $name, "-fill.svg"),
                    None,
                )
            };
            ($name:literal, second) => {
                (
                    concat!("captain/icons/", $name, "-line.svg"),
                    concat!("captain/icons/", $name, "-fill.svg"),
                    Some(concat!("captain/icons/", $name, "-fill2.svg")),
                )
            };
        }
        match self {
            CaptainIcon::Container => files!("container"),
            CaptainIcon::Image => files!("image", second),
            CaptainIcon::Volume => files!("volume", second),
            CaptainIcon::Network => files!("network"),
            CaptainIcon::Stack => files!("stack"),
            CaptainIcon::Pod => files!("pod", second),
            CaptainIcon::Cluster => files!("cluster", second),
            CaptainIcon::Snapshot => files!("snapshot"),
            CaptainIcon::Extension => files!("extension"),
            CaptainIcon::Engine => files!("engine", second),
            CaptainIcon::Forward => files!("forward"),
            CaptainIcon::Exec => files!("exec"),
            CaptainIcon::Reclaim => files!("reclaim"),
        }
    }
}
