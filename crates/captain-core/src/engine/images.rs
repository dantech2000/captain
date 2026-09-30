use std::time::Duration;

use super::{EngineFuture, EngineStream};
use crate::model::{Image, ImageDetail, ImageLayer, PullProgress, RunSpec, ScanProgress};

/// Images: list, inspect, remove, prune, pull, run, tag, push, and scan.
pub trait ImageApi {
    /// All images, including untagged ones, with the number of containers that use each.
    fn list_images(&self) -> EngineFuture<Vec<Image>>;

    /// Details of one image: tags, digests, platform, and its run defaults.
    fn inspect_image(&self, id: &str) -> EngineFuture<ImageDetail>;

    /// The build steps of one image, newest first.
    fn image_history(&self, id: &str) -> EngineFuture<Vec<ImageLayer>>;

    /// Removes one image by ID, or one tag by reference. It never forces, so it fails
    /// if a container uses the image.
    fn remove_image(&self, id: &str) -> EngineFuture<()>;

    /// Removes every dangling (untagged, unused) image. Returns the bytes reclaimed.
    fn prune_dangling_images(&self) -> EngineFuture<u64>;

    /// Pulls `reference`, for example `busybox` or `nginx:1.27`. A missing tag means
    /// `latest`. The stream ends when the pull is done, or after one error.
    fn pull_image(&self, reference: &str) -> EngineStream<PullProgress>;

    /// Creates a container from `spec` and starts it, like `docker run -d`. Returns the
    /// new container's ID. If the start fails, the created container stays.
    fn run_container(&self, spec: RunSpec) -> EngineFuture<String>;

    /// Adds the tag `target`, for example `ghcr.io/team/app:1.0`, to the image
    /// `source`, an ID or a reference. A missing tag means `latest`.
    fn tag_image(&self, source: &str, target: &str) -> EngineFuture<()>;

    /// Pushes the tag `reference` with the login from the Docker config. The stream
    /// ends when the push is done, or after one error. A registry that wants a login
    /// fails with a hint to run `docker login`.
    fn push_image(&self, reference: &str) -> EngineStream<PullProgress>;

    /// Scans `reference` for vulnerabilities with Trivy, run as a container. The
    /// stream yields status lines, then one report, or one error. Dropping the stream
    /// stops the scan.
    fn scan_image(&self, reference: &str) -> EngineStream<ScanProgress>;

    /// Removes build cache that no build used for `older_than`, like
    /// `docker builder prune --filter until=336h`. Internal and frontend records stay.
    /// Returns the bytes reclaimed.
    fn prune_build_cache(&self, older_than: Duration) -> EngineFuture<u64>;

    /// Removes the one build cache record `id` if no build used it for `older_than`,
    /// like [`prune_build_cache`](Self::prune_build_cache) with an `id` filter. No
    /// other record goes. Returns the bytes reclaimed.
    fn prune_build_record(&self, id: &str, older_than: Duration) -> EngineFuture<u64>;
}
