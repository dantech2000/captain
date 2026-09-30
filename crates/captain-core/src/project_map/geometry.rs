//! The shapes a map layout is made of, in map units: 1 is one pixel at 100%.

pub const PIN_WIDTH: f32 = 100.;
pub const PIN_HEIGHT: f32 = 30.;
pub const NODE_WIDTH: f32 = 230.;
pub const NODE_HEIGHT: f32 = 104.;
pub const VOLUME_WIDTH: f32 = 200.;
pub const VOLUME_HEIGHT: f32 = 64.;

/// A rectangle in map units: 1 is one pixel at 100%.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub(super) fn mid_y(&self) -> f32 {
        self.y + self.h / 2.
    }

    pub(super) fn right(&self) -> f32 {
        self.x + self.w
    }

    pub(super) fn bottom(&self) -> f32 {
        self.y + self.h
    }
}

/// A node on the map: a service by its container ID, or a volume by its name.
#[derive(Debug, Clone, PartialEq)]
pub struct Placed {
    pub key: String,
    pub rect: Rect,
}

/// A network's dashed lane around its services.
#[derive(Debug, Clone, PartialEq)]
pub struct Lane {
    /// The network's name. Empty for services with no network yet.
    pub network: String,
    pub rect: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeKind {
    /// From a host port to its service.
    Port,
    /// From a service to one its environment names.
    TalksTo,
    /// From a service to a volume it mounts.
    Mount,
}

/// A cubic Bézier curve from `from` to `to`, bent by two control points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Edge {
    pub kind: EdgeKind,
    pub from: (f32, f32),
    pub c1: (f32, f32),
    pub c2: (f32, f32),
    pub to: (f32, f32),
}

/// Where everything goes, in map units.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MapLayout {
    pub width: f32,
    pub height: f32,
    /// The top left of the "This Mac" and "Volumes" labels.
    pub pins_label: (f32, f32),
    pub volumes_label: Option<(f32, f32)>,
    /// Host ports, with the ID of the container each reaches.
    pub pins: Vec<(u16, String, Rect)>,
    pub lanes: Vec<Lane>,
    pub nodes: Vec<Placed>,
    pub volumes: Vec<Placed>,
    pub edges: Vec<Edge>,
}

impl MapLayout {
    /// The rectangle of the service with container ID `id`.
    pub fn node(&self, id: &str) -> Option<Rect> {
        self.nodes.iter().find(|n| n.key == id).map(|n| n.rect)
    }
}
