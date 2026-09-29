use super::EngineStream;
use crate::model::BuildSpec;

/// Builds images. The Engine API's `/build` needs a context tar and a BuildKit
/// session, so this is separate from [`Engine`](crate::Engine): the Docker
/// implementation runs `docker buildx build`. See
/// docs/features/0019-image-build-push-scan.md.
pub trait ImageBuilder: Send + Sync + 'static {
    /// Builds `spec` and tags the result. The stream yields the builder's output
    /// lines. It ends after a successful build, or with one error. Dropping the
    /// stream stops the build.
    fn build(&self, spec: &BuildSpec) -> EngineStream<String>;
}
