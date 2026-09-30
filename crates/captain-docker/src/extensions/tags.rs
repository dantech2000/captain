//! The tag to pull when the user names none: the newest version tag on the registry
//! or in the engine, else `latest`. See `captain_core::extension::choose_tag`.

use bollard::Docker;
use bollard::query_parameters::ListImagesOptionsBuilder;
use captain_core::extension::{choose_tag, image_repository};

use super::registry;

/// The tag for `repository`. The engine's own tags count too, so a locally built
/// extension resolves without a registry. A registry that cannot be reached only
/// logs.
pub async fn resolve(docker: &Docker, repository: &str) -> String {
    let mut tags = match registry::tags(repository).await {
        Ok(tags) => tags,
        Err(error) => {
            tracing::info!(%error, %repository, "cannot list the extension's tags");
            Vec::new()
        }
    };
    tags.extend(local_tags(docker, repository).await);
    let tag = choose_tag(&tags);
    tracing::debug!(%repository, %tag, count = tags.len(), "picked the extension tag");
    tag
}

/// The tags of `repository` that the engine has.
async fn local_tags(docker: &Docker, repository: &str) -> Vec<String> {
    let options = ListImagesOptionsBuilder::default().build();
    let Ok(images) = docker.list_images(Some(options)).await else {
        return Vec::new();
    };
    images
        .into_iter()
        .flat_map(|image| image.repo_tags)
        .filter_map(|reference| {
            let (_, tag) = reference.rsplit_once(':')?;
            (image_repository(&reference).as_deref() == Some(repository)).then(|| tag.to_string())
        })
        .collect()
}
