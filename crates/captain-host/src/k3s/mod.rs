//! k3s on the host side: the version list and the checked downloads in
//! `~/.captain/cache`. Nothing downloads inside the VM. See ADR 0010.

mod curl;
mod download;
mod versions;

pub use curl::ping;
pub use download::ensure;
pub use versions::list;
