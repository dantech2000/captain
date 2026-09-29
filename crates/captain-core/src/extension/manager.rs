use futures::stream::BoxStream;

use super::{BridgeEvent, BridgeRequest, ExtensionCandidate, ExtensionPaths, InstalledExtension};
use crate::EngineFuture;

/// The answers to one bridge call. It ends after the last event.
pub type BridgeStream = BoxStream<'static, BridgeEvent>;

/// Installs, lists, and removes extensions on the connected engine, and answers
/// the calls their pages make. Like [`crate::Engine`], the futures and streams must
/// not depend on a specific async runtime. See docs/adr/0011-extensions.md.
pub trait ExtensionManager: Send + Sync + 'static {
    /// The installed extensions, from `~/.captain/extensions`, sorted by title.
    fn list(&self) -> EngineFuture<Vec<InstalledExtension>>;

    /// Pulls `reference` when the engine does not have it, and reads its labels and
    /// `metadata.json`. Fails when the image is not an extension. Nothing runs yet.
    fn prepare(&self, reference: &str) -> EngineFuture<ExtensionCandidate>;

    /// Copies the UI and the host binaries for this platform out of the image, starts
    /// the backend, and records the extension. A failed install removes what it made.
    fn install(&self, candidate: ExtensionCandidate) -> EngineFuture<InstalledExtension>;

    /// Removes the backend with its volumes, the extension's folder, and its image.
    fn remove(&self, extension: InstalledExtension) -> EngineFuture<()>;

    fn paths(&self) -> &ExtensionPaths;

    /// Answers a request that [`BridgeRequest::route`] sends to the engine side.
    /// Dropping the stream stops the call, and kills a running command.
    fn call(&self, extension: &InstalledExtension, request: BridgeRequest) -> BridgeStream;
}
