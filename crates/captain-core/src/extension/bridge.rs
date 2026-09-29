//! The bridge between an extension's page and Captain: the `ddClient` shim, the
//! messages it posts, and the replies. See docs/adr/0011-extensions.md.

mod host_binary;
mod http;
mod options;
mod reply;
mod request;
mod shim;

pub use host_binary::host_binary;
pub use http::{parse_response, request_bytes};
pub use options::{ListOptions, OpenDialogOptions};
pub use reply::{BridgeEvent, exec_result, service_result};
pub use request::{
    BridgeCall, BridgeError, BridgeRequest, ExecRequest, ExecScope, Route, ServiceRequest,
    ToastLevel, parse_call, unquote,
};
pub use shim::{ShimContext, init_script};
