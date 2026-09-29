/// The folder that holds Captain Engine's Lima instance. See
/// docs/adr/0008-captain-engine.md.
const CAPTAIN_LIMA_DIR: &str = "/Captain/lima/";

/// True if `endpoint` is Captain Engine's own socket. The assistant copies into any
/// engine, but it says so when the target is not Captain Engine.
pub fn is_captain_engine(endpoint: &str) -> bool {
    endpoint.starts_with("unix://") && endpoint.contains(CAPTAIN_LIMA_DIR)
}

#[cfg(test)]
mod tests;
