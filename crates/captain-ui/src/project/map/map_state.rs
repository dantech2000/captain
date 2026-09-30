use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use captain_core::model::RestartPolicy;
use gpui_kit::*;

/// How large the map draws.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Zoom {
    /// As wide as the page, never larger than 100%.
    #[default]
    Fit,
    /// 100%: one map unit is one pixel.
    Actual,
}

/// The values the editor on a node shows before they are staged.
#[derive(Debug, Clone, PartialEq)]
pub struct Draft {
    pub id: String,
    pub name: String,
    /// The memory limit in bytes. 0 means no limit.
    pub memory: u64,
    /// The CPU limit in billionths of a CPU. 0 means no limit.
    pub nano_cpus: u64,
    pub restart: RestartPolicy,
}

/// The Map tab's own state.
#[derive(Default)]
pub struct MapState {
    pub zoom: Zoom,
    /// The width of the map area at the last paint, for Fit.
    pub width: Rc<Cell<f32>>,
    /// The node whose editor is open.
    pub draft: Option<Draft>,
    /// Volume sizes from the volume list, by name.
    pub volume_sizes: HashMap<String, u64>,
    pub volumes_task: Option<Task<()>>,
    /// Containers whose staged changes are being applied.
    pub applying: HashSet<String>,
}

impl MapState {
    /// The scale to draw a map `map_width` wide at.
    pub fn scale(&self, map_width: f32) -> f32 {
        match self.zoom {
            Zoom::Actual => 1.,
            Zoom::Fit => {
                let width = self.width.get();
                if width <= 0. || map_width <= 0. {
                    1.
                } else {
                    (width / map_width).clamp(0.4, 1.)
                }
            }
        }
    }
}
