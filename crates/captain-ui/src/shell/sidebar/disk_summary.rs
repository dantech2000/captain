use captain_core::storage::DiskBreakdown;
use gpui_kit::*;

use crate::storage::storage_model;

/// What the sidebar status line shows of the engine disk: the storage model's disk
/// use, and the bytes that the cleanup groups checked by default free.
pub struct DiskSummary {
    pub breakdown: DiskBreakdown,
    pub freeable: u64,
}

impl DiskSummary {
    /// The disk use, once the storage model has read it.
    pub fn read(cx: &App) -> Option<Self> {
        let model = storage_model(cx)?;
        let model = model.read(cx);
        Some(Self {
            breakdown: model.breakdown(cx)?,
            freeable: model.default_bytes(),
        })
    }
}
