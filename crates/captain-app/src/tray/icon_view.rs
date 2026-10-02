//! Puts the [`IconLook`] on the menu bar icon: turns the wheel, and follows the
//! `menu_bar_status_dot` setting.

use std::time::Duration;

use gpui_kit::*;
use tray_icon::Icon;

use super::controller::{Tray, TrayHandle};
use super::icon;
use super::look::{Dot, IconLook, Wheel};

/// How long each frame of the turning wheel shows.
const TURN_FRAME: Duration = Duration::from_millis(120);

/// Template images on macOS take the menu bar's color, so only the alpha counts.
/// The Windows taskbar is dark by default, so the icon is white there.
const COLOR: [u8; 3] = if cfg!(target_os = "macos") {
    [0, 0, 0]
} else {
    [255, 255, 255]
};

/// Turns the stop-light dot on or off, after a change to the setting.
pub fn set_status_dot(colored: bool, cx: &mut App) {
    if let Some(tray) = cx.try_global::<TrayHandle>().map(|handle| handle.0.clone()) {
        tray.update(cx, |tray, cx| {
            if tray.colored != colored {
                tray.colored = colored;
                if let Some(look) = tray.shown.as_ref().map(|shown| shown.look(colored)) {
                    tray.show_look(look, cx);
                }
            }
        });
    }
}

/// The icon with every dot painted into the image: a template on macOS, which
/// suits a plain look only.
pub fn plain_icon(look: IconLook) -> Icon {
    rgba_icon(look, 0, true)
}

fn rgba_icon(look: IconLook, frame: u32, paint_dot: bool) -> Icon {
    Icon::from_rgba(
        icon::rgba(look, frame, COLOR, paint_dot),
        icon::SIZE,
        icon::SIZE,
    )
    .expect("the icon buffer matches its size")
}

impl Tray {
    /// Shows `look`, unless the icon shows it already, and turns the wheel while
    /// the look says so.
    pub(super) fn show_look(&mut self, look: IconLook, cx: &mut Context<Self>) {
        if self.look == Some(look) {
            return;
        }
        self.look = Some(look);
        self.set_icon(look, 0);
        self.spin = (look.wheel == Wheel::Turning).then(|| Self::turn(look, cx));
    }

    /// On macOS a colored dot cannot be part of a template image, so AppKit
    /// draws the template wheel and the dot together; see [`super::status_dot`].
    fn set_icon(&self, look: IconLook, frame: u32) {
        let colored = match look.dot {
            Some(Dot::Colored(light)) => Some(light),
            _ => None,
        };
        let paint_dot = cfg!(not(target_os = "macos")) || colored.is_none();
        let set = self
            .icon
            .set_icon_with_as_template(Some(rgba_icon(look, frame, paint_dot)), true);
        match set {
            Err(error) => tracing::warn!(%error, "cannot update the menu bar icon"),
            #[cfg(target_os = "macos")]
            Ok(()) => {
                if let Some(light) = colored {
                    super::status_dot::paint(&self.icon, light);
                }
            }
            #[cfg(not(target_os = "macos"))]
            Ok(()) => {}
        }
    }

    /// Advances the wheel one frame at a time until the task is dropped.
    fn turn(look: IconLook, cx: &mut Context<Self>) -> Task<()> {
        cx.spawn(async move |this, cx| {
            let mut frame = 0;
            loop {
                cx.background_executor().timer(TURN_FRAME).await;
                frame = (frame + 1) % icon::TURN_FRAMES;
                if this
                    .update(cx, |tray, _| tray.set_icon(look, frame))
                    .is_err()
                {
                    return;
                }
            }
        })
    }
}
