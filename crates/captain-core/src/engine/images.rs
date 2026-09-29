use super::{EngineFuture, EngineStream};
use crate::model::{Image, ImageDetail, ImageLayer, PullProgress, RunSpec};

/// Images: list, inspect, remove, prune, pull, and run.
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
}
