//! Names derived from an extension's image: its ID, its Compose project, and its web
//! data store.

use crate::model::ImageReference;

/// The prefix of every extension's Compose project.
pub const PROJECT_PREFIX: &str = "captain-ext-";

/// The extension ID for an image reference: the repository without tag or digest,
/// in lowercase, with every other character than a letter or digit as `-`, then `-`
/// and 8 hex digits of a hash of the repository. The hash keeps `acme/foo-bar` and
/// `acme/foo_bar` apart. The ID names the folder in `~/.captain/extensions` and the
/// host of the `captain-ext://` URL, so it is a valid host name. `None` for input
/// that is not an image reference.
pub fn extension_id(reference: &str) -> Option<String> {
    let repository = image_repository(reference)?;
    let repository = repository.as_str();
    let mut id = String::new();
    for c in repository.chars() {
        let c = c.to_ascii_lowercase();
        if c.is_ascii_alphanumeric() {
            id.push(c);
        } else if !id.is_empty() && !id.ends_with('-') {
            id.push('-');
        }
    }
    let id = id.trim_end_matches('-');
    let hash = fnv1a(repository, FNV_OFFSET) >> 32;
    (!id.is_empty()).then(|| format!("{id}-{hash:08x}"))
}

/// The repository of an image reference, without tag or digest: `acme/foo:1` is
/// `acme/foo`. `None` for input that is not an image reference.
pub fn image_repository(reference: &str) -> Option<String> {
    let parsed = ImageReference::parse(reference)?;
    Some(
        parsed
            .name
            .split('@')
            .next()
            .unwrap_or_default()
            .to_string(),
    )
}

/// The Compose project that runs the backend of the extension `id`.
pub fn project_name(id: &str) -> String {
    format!("{PROJECT_PREFIX}{id}")
}

/// A fixed 16-byte value for the extension `id`, so each extension keeps its own
/// cookies and storage in WebKit (`WKWebsiteDataStore(forIdentifier:)`). Two
/// FNV-1a hashes with different offsets make the 16 bytes.
pub fn data_store_id(id: &str) -> [u8; 16] {
    let mut bytes = [0; 16];
    bytes[..8].copy_from_slice(&fnv1a(id, FNV_OFFSET).to_be_bytes());
    bytes[8..].copy_from_slice(&fnv1a(id, 0x6c62_272e_07bb_0142).to_be_bytes());
    bytes
}

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;

/// The 64-bit FNV-1a hash of `text`, starting from `offset`.
fn fnv1a(text: &str, offset: u64) -> u64 {
    const PRIME: u64 = 0x0100_0000_01b3;
    text.bytes().fold(offset, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(PRIME)
    })
}

#[cfg(test)]
mod tests;
