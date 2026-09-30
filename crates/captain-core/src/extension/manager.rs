use futures::stream::BoxStream;

use super::{
    BridgeEvent, BridgeRequest, ExtensionCandidate, ExtensionPaths, InstalledExtension, UpdateCheck,
};
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
    /// `metadata.json`. A reference without a tag gets the newest version tag of the
    /// repository. Fails when the image is not an extension, and then removes an
    /// image it pulled. Nothing runs yet.
    fn prepare(&self, reference: &str) -> EngineFuture<ExtensionCandidate>;

    /// Removes the image of a candidate the user did not install, if `prepare` or
    /// `check_update` pulled it.
    fn discard(&self, candidate: ExtensionCandidate) -> EngineFuture<()>;

    /// Copies the UI and the host binaries for this platform out of the image, starts
    /// the backend, and records the extension. A failed install removes what it made.
    fn install(&self, candidate: ExtensionCandidate) -> EngineFuture<InstalledExtension>;

    /// The newest version tag of the extension's repository, on its registry or in
    /// the engine, else `latest`. The Update dialog offers it.
    fn newest_tag(&self, extension: InstalledExtension) -> EngineFuture<String>;

    /// Pulls the extension's repository with `tag` (the newest tag when empty) and compares
    /// the image with the installed one. A failed pull uses the engine's copy of that
    /// image, if it has one, so a locally built extension can update too.
    fn check_update(&self, extension: InstalledExtension, tag: String)
    -> EngineFuture<UpdateCheck>;

    /// Replaces the extension's files with the new image's, and restarts its backend
    /// on the new image. The backend's volumes and other files in the extension's
    /// folder stay.
    fn update(
        &self,
        extension: InstalledExtension,
        candidate: ExtensionCandidate,
    ) -> EngineFuture<InstalledExtension>;

    /// Removes the backend with its volumes, the extension's folder, and its image.
    fn remove(&self, extension: InstalledExtension) -> EngineFuture<()>;

    fn paths(&self) -> &ExtensionPaths;

    /// Answers a request that [`BridgeRequest::route`] sends to the engine side.
    /// Dropping the stream stops the call, and kills a running command.
    fn call(&self, extension: &InstalledExtension, request: BridgeRequest) -> BridgeStream;
}
