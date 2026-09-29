//! Names derived from an extension's image: its ID, its Compose project, and its web
//! data store.

use crate::model::ImageReference;

/// The prefix of every extension's Compose project.
pub const PROJECT_PREFIX: &str = "captain-ext-";

/// The extension ID for an image reference: the repository without tag or digest,
/// in lowercase, with every other character than a letter or digit as `-`. It names
/// the folder in `~/.captain/extensions` and the host of the `captain-ext://` URL, so
/// it is a valid host name. `None` for input that is not an image reference.
pub fn extension_id(reference: &str) -> Option<String> {
    let parsed = ImageReference::parse(reference)?;
    let repository = parsed.name.split('@').next().unwrap_or_default();
    let mut id = String::new();
    for c in repository.chars() {
        let c = c.to_ascii_lowercase();
        if c.is_ascii_alphanumeric() {
            id.push(c);
        } else if !id.is_empty() && !id.ends_with('-') {
            id.push('-');
        }
    }
    let id = id.trim_end_matches('-').to_string();
    (!id.is_empty()).then_some(id)
}

/// The Compose project that runs the backend of the extension `id`.
pub fn project_name(id: &str) -> String {
    format!("{PROJECT_PREFIX}{id}")
}

/// A fixed 16-byte value for the extension `id`, so each extension keeps its own
/// cookies and storage in WebKit (`WKWebsiteDataStore(forIdentifier:)`). Two
/// FNV-1a hashes with different offsets make the 16 bytes.
pub fn data_store_id(id: &str) -> [u8; 16] {
    const PRIME: u64 = 0x0100_0000_01b3;
    let hash = |offset: u64| {
        id.bytes().fold(offset, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(PRIME)
        })
    };
    let mut bytes = [0; 16];
    bytes[..8].copy_from_slice(&hash(0xcbf2_9ce4_8422_2325).to_be_bytes());
    bytes[8..].copy_from_slice(&hash(0x6c62_272e_07bb_0142).to_be_bytes());
    bytes
}

#[cfg(test)]
mod tests;
