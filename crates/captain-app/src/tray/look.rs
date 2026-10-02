//! What the menu bar icon shows: the ship's wheel, and a stop-light dot on it.
//! Green while the engine runs and nothing is wrong, amber while it starts, stops,
//! or reconnects or while a container is unhealthy, red when it does not answer, a
//! check fails, or a container keeps restarting (the menu's problem line uses the
//! same light), and no dot with a dimmed wheel while it is stopped. See feature 0009.

use captain_ui::EngineHealth;

use super::dot::Light;
use super::problem_menu::problem_light;
use super::snapshot::TraySnapshot;

/// How the wheel draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wheel {
    Full,
    /// It turns while the engine starts, stops, or reconnects.
    Turning,
    /// Dimmed, like a macOS menu bar icon that is off.
    Dim,
}

/// The dot on the wheel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dot {
    /// In the wheel's own color, for the plain icon (`menu_bar_status_dot` off).
    Plain,
    /// In a stop-light color.
    Colored(Light),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IconLook {
    pub wheel: Wheel,
    pub dot: Option<Dot>,
}

impl TraySnapshot {
    /// The stop light for the icon, or `None` while the engine is stopped.
    pub fn icon_light(&self) -> Option<Light> {
        match self.engine {
            // The same light as the problem's line in the menu.
            EngineHealth::Running => {
                Some(self.problem.as_ref().map_or(Light::Green, problem_light))
            }
            EngineHealth::Connecting | EngineHealth::Starting | EngineHealth::Reconnecting => {
                Some(Light::Amber)
            }
            EngineHealth::NotAnswering | EngineHealth::CannotRun => Some(Light::Red),
            EngineHealth::Stopped | EngineHealth::NotSetUp => None,
        }
    }

    /// The icon. With `colored` off it is a plain template image, as before the
    /// stop light: a plain dot only when something is wrong.
    pub fn look(&self, colored: bool) -> IconLook {
        let light = self.icon_light();
        let wheel = match light {
            None => Wheel::Dim,
            Some(Light::Amber) if self.engine != EngineHealth::Running => Wheel::Turning,
            Some(_) => Wheel::Full,
        };
        let dot = match light {
            Some(light) if colored => Some(Dot::Colored(light)),
            Some(Light::Red) => Some(Dot::Plain),
            Some(Light::Amber) if wheel == Wheel::Full => Some(Dot::Plain),
            _ => None,
        };
        IconLook { wheel, dot }
    }
}

#[cfg(test)]
mod tests;
