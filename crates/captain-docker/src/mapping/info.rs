use bollard::models::SystemVersion;
use captain_core::model::EngineInfo;

use crate::Endpoint;

pub fn engine_info(version: SystemVersion, endpoint: &Endpoint) -> EngineInfo {
    EngineInfo {
        version: version.version.unwrap_or_default(),
        api_version: version.api_version.unwrap_or_default(),
        os: version.os.unwrap_or_default(),
        arch: version.arch.unwrap_or_default(),
        endpoint: endpoint.to_string(),
    }
}
