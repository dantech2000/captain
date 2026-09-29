use captain_core::model::VolumeUser;
use gpui_kit::*;

use super::VolumesView;

/// The containers that use one volume, or why they could not be loaded.
pub struct VolumeUsers {
    /// The volume the result belongs to.
    pub name: String,
    pub result: Result<Vec<VolumeUser>, String>,
}

impl VolumesView {
    /// Loads the containers that use the selected volume. A newer call cancels a
    /// pending one. The old result stays until the new one arrives, so the panel does
    /// not flicker on a reload.
    pub(super) fn load_users(&mut self, cx: &mut Context<Self>) {
        let (Some(engine), Some(name)) = (self.engine.clone(), self.selected.clone()) else {
            self.users = None;
            self.users_task = None;
            return;
        };
        self.users_task = Some(cx.spawn(async move |this, cx| {
            let result = engine.volume_users(&name).await;
            this.update(cx, |this, cx| {
                if this.selected.as_deref() != Some(name.as_str()) {
                    return;
                }
                let result = result.map_err(|error| {
                    tracing::warn!(%error, volume = %name, "could not list volume users");
                    error.to_string()
                });
                this.users = Some(VolumeUsers { name, result });
                cx.notify();
            })
            .ok();
        }));
    }

    /// The loaded users of the volume `name`, if they belong to it.
    pub(super) fn users_of(&self, name: &str) -> Option<&Result<Vec<VolumeUser>, String>> {
        self.users
            .as_ref()
            .filter(|users| users.name == name)
            .map(|users| &users.result)
    }
}
