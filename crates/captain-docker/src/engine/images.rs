//! Images: list, inspect, remove, prune, pull, and run. Every call runs on the private
//! tokio runtime.

mod run;

use std::collections::HashMap;
use std::pin::pin;

use bollard::query_parameters::{
    CreateImageOptionsBuilder, ListContainersOptionsBuilder, ListImagesOptionsBuilder,
    PruneImagesOptionsBuilder, RemoveImageOptions,
};
use captain_core::model::{Image, ImageDetail, ImageLayer, ImageReference, PullProgress, RunSpec};
use captain_core::{EngineError, EngineFuture, EngineStream, ImageApi};
use futures::StreamExt;

use super::DockerEngine;
use crate::{mapping, runtime};

impl ImageApi for DockerEngine {
    fn list_images(&self) -> EngineFuture<Vec<Image>> {
        let docker = self.docker.clone();
        runtime::spawn(self.runtime.handle(), async move {
            let options = ListImagesOptionsBuilder::default().all(false).build();
            let summaries = docker
                .list_images(Some(options))
                .await
                .map_err(mapping::engine_error)?;
            // The engine often reports -1 containers. Then count them from the container list.
            let usage = if mapping::needs_usage(&summaries) {
                let options = ListContainersOptionsBuilder::default().all(true).build();
                let containers = docker
                    .list_containers(Some(options))
                    .await
                    .map_err(mapping::engine_error)?;
                mapping::image_usage(&containers)
            } else {
                HashMap::new()
            };
            Ok(summaries
                .into_iter()
                .map(|summary| mapping::image(summary, &usage))
                .collect())
        })
    }

    fn inspect_image(&self, id: &str) -> EngineFuture<ImageDetail> {
        let docker = self.docker.clone();
        let id = id.to_string();
        runtime::spawn(self.runtime.handle(), async move {
            let inspect = docker
                .inspect_image(&id)
                .await
                .map_err(mapping::engine_error)?;
            Ok(mapping::image_detail(inspect))
        })
    }

    fn image_history(&self, id: &str) -> EngineFuture<Vec<ImageLayer>> {
        let docker = self.docker.clone();
        let id = id.to_string();
        runtime::spawn(self.runtime.handle(), async move {
            mapping::image_history(docker.image_history(&id).await)
        })
    }

    fn remove_image(&self, id: &str) -> EngineFuture<()> {
        let docker = self.docker.clone();
        let id = id.to_string();
        runtime::spawn(self.runtime.handle(), async move {
            docker
                .remove_image(&id, None::<RemoveImageOptions>, None)
                .await
                .map(|_| ())
                .map_err(mapping::engine_error)
        })
    }

    fn prune_dangling_images(&self) -> EngineFuture<u64> {
        let docker = self.docker.clone();
        runtime::spawn(self.runtime.handle(), async move {
            let filters = HashMap::from([("dangling", vec!["true"])]);
            let options = PruneImagesOptionsBuilder::default()
                .filters(&filters)
                .build();
            let response = docker
                .prune_images(Some(options))
                .await
                .map_err(mapping::engine_error)?;
            Ok(response
                .space_reclaimed
                .and_then(|bytes| u64::try_from(bytes).ok())
                .unwrap_or(0))
        })
    }

    fn pull_image(&self, reference: &str) -> EngineStream<PullProgress> {
        let docker = self.docker.clone();
        let input = reference.to_string();
        runtime::forward(self.runtime.handle(), move |tx| async move {
            let Some(reference) = ImageReference::parse(&input) else {
                let error = EngineError::Api(format!("invalid reference format: {input:?}"));
                tx.unbounded_send(Err(error)).ok();
                return;
            };
            let mut options = CreateImageOptionsBuilder::default().from_image(&reference.name);
            if !reference.tag.is_empty() {
                options = options.tag(&reference.tag);
            }
            let mut messages = pin!(docker.create_image(Some(options.build()), None, None));
            while let Some(item) = messages.next().await {
                let failed = item.is_err();
                let item = item
                    .map(mapping::pull_progress)
                    .map_err(mapping::pull_error);
                if tx.unbounded_send(item).is_err() || failed {
                    break;
                }
            }
        })
    }

    fn run_container(&self, spec: RunSpec) -> EngineFuture<String> {
        let docker = self.docker.clone();
        runtime::spawn(self.runtime.handle(), async move {
            run::run_container(&docker, spec).await
        })
    }
}
