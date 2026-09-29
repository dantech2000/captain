//! The Scan dialog: runs Trivy on one image and lists the vulnerabilities, with a
//! severity filter.

mod list;
mod scan_state;
mod scan_view;

pub use scan_state::ScanDialog;
pub use scan_view::open;
