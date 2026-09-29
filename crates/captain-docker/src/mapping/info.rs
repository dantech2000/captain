use bollard::models::{SystemInfo, SystemVersion};
use captain_core::model::EngineInfo;

pub fn engine_info(version: SystemVersion, info: SystemInfo, endpoint: String) -> EngineInfo {
    EngineInfo {
        version: version.version.unwrap_or_default(),
        api_version: version.api_version.unwrap_or_default(),
        os: version.os.unwrap_or_default(),
        arch: version.arch.unwrap_or_default(),
        endpoint,
        cpus: info.ncpu.unwrap_or_default().try_into().unwrap_or_default(),
        memory_bytes: info
            .mem_total
            .unwrap_or_default()
            .try_into()
            .unwrap_or_default(),
    }
}
