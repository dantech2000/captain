use std::sync::Arc;

use captain_core::Engine;
use captain_core::model::{ScanProgress, ScanReport, Severity};
use futures::StreamExt;
use gpui_kit::*;

/// Where the scan is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanStatus {
    /// Running, with Trivy's latest log message.
    Running(String),
    Done(ScanReport),
    Failed(String),
}

/// One scan and the severity filter for its results.
pub struct ScanDialog {
    pub(super) reference: String,
    pub(super) status: ScanStatus,
    /// `None` shows every severity.
    pub(super) filter: Option<Severity>,
    pub(super) scroll: UniformListScrollHandle,
    /// Dropping it drops the scan stream, which stops the scan.
    _task: Task<()>,
}

impl ScanDialog {
    /// Starts scanning `reference` right away.
    pub fn new(engine: Arc<dyn Engine>, reference: String, cx: &mut Context<Self>) -> Self {
        let mut messages = engine.scan_image(&reference);
        let task = cx.spawn(async move |this, cx| {
            while let Some(message) = messages.next().await {
                let updated = this.update(cx, |this, cx| {
                    this.status = match message {
                        Ok(ScanProgress::Status(status)) => ScanStatus::Running(status),
                        Ok(ScanProgress::Report(report)) => ScanStatus::Done(report),
                        Err(error) => {
                            tracing::warn!(%error, "scanning an image failed");
                            ScanStatus::Failed(format!("Scan failed: {error}"))
                        }
                    };
                    cx.notify();
                });
                if updated.is_err() {
                    break;
                }
            }
        });
        Self {
            reference,
            status: ScanStatus::Running("Starting Trivy".into()),
            filter: None,
            scroll: UniformListScrollHandle::new(),
            _task: task,
        }
    }

    pub(super) fn set_filter(&mut self, filter: Option<Severity>, cx: &mut Context<Self>) {
        self.filter = filter;
        cx.notify();
    }
}
