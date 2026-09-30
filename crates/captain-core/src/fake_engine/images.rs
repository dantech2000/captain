use std::time::Duration;

use futures::future::ready;
use futures::stream;
use futures::{FutureExt, StreamExt};

use super::FakeEngine;
use crate::model::{
    Image, ImageDetail, ImageLayer, ImageReference, PullProgress, RunSpec, ScanProgress, ScanReport,
};
use crate::{EngineError, EngineFuture, EngineStream, ImageApi};

/// The ID every run on the fake engine returns.
const FAKE_RUN_ID: &str = "fake-run";

/// The fake engine's images data.
#[derive(Debug, Clone, Default)]
pub struct FakeImages {
    pub images: Vec<Image>,
    /// Details by image ID. Inspecting an image missing here fails.
    pub details: Vec<ImageDetail>,
    /// The history every image reports.
    pub history: Vec<ImageLayer>,
    /// The messages every pull and push streams.
    pub pull: Vec<PullProgress>,
    /// The report every scan ends with. `None` makes scans fail.
    pub scan: Option<ScanReport>,
}

impl ImageApi for FakeEngine {
    fn list_images(&self) -> EngineFuture<Vec<Image>> {
        ready(Ok(self.images.images.clone())).boxed()
    }

    fn inspect_image(&self, id: &str) -> EngineFuture<ImageDetail> {
        let result = self
            .images
            .details
            .iter()
            .find(|detail| detail.id == id)
            .cloned()
            .ok_or_else(|| no_such_image(id));
        ready(result).boxed()
    }

    fn image_history(&self, id: &str) -> EngineFuture<Vec<ImageLayer>> {
        let result = if self.images.images.iter().any(|image| image.id == id) {
            Ok(self.images.history.clone())
        } else {
            Err(no_such_image(id))
        };
        ready(result).boxed()
    }

    fn remove_image(&self, id: &str) -> EngineFuture<()> {
        let result = match self.images.images.iter().find(|image| image.id == id) {
            None => Err(no_such_image(id)),
            Some(image) if image.in_use() => Err(EngineError::Api(format!(
                "conflict: unable to delete {} - image is being used",
                image.short_id()
            ))),
            Some(_) => Ok(()),
        };
        ready(result).boxed()
    }

    fn prune_dangling_images(&self) -> EngineFuture<u64> {
        let reclaimed = self
            .images
            .images
            .iter()
            .filter(|image| image.dangling && !image.in_use())
            .map(|image| image.size)
            .sum();
        ready(Ok(reclaimed)).boxed()
    }

    fn pull_image(&self, reference: &str) -> EngineStream<PullProgress> {
        if ImageReference::parse(reference).is_none() {
            let error = EngineError::Api(format!("invalid reference format: {reference:?}"));
            return stream::once(ready(Err(error))).boxed();
        }
        stream::iter(self.images.pull.clone().into_iter().map(Ok)).boxed()
    }

    /// Accepts a spec for a known image, by ID or tag, and a name no container has.
    fn run_container(&self, spec: RunSpec) -> EngineFuture<String> {
        let known = self
            .images
            .images
            .iter()
            .any(|image| image.id == spec.image || image.repo_tags.contains(&spec.image));
        let taken = spec
            .name
            .as_ref()
            .is_some_and(|name| self.containers.iter().any(|c| &c.name == name));
        let result = if !known {
            Err(no_such_image(&spec.image))
        } else if taken {
            Err(EngineError::Api(format!(
                "Conflict. The container name \"/{}\" is already in use",
                spec.name.unwrap_or_default()
            )))
        } else {
            Ok(FAKE_RUN_ID.to_string())
        };
        ready(result).boxed()
    }

    fn tag_image(&self, source: &str, target: &str) -> EngineFuture<()> {
        let result = if !self.images.known(source) {
            Err(no_such_image(source))
        } else if ImageReference::parse(target).is_none_or(|r| r.tag.is_empty()) {
            Err(EngineError::Api(format!(
                "invalid reference format: {target:?}"
            )))
        } else {
            Ok(())
        };
        ready(result).boxed()
    }

    fn push_image(&self, reference: &str) -> EngineStream<PullProgress> {
        if !self.images.known(reference) {
            return stream::once(ready(Err(no_such_image(reference)))).boxed();
        }
        stream::iter(self.images.pull.clone().into_iter().map(Ok)).boxed()
    }

    fn scan_image(&self, reference: &str) -> EngineStream<ScanProgress> {
        let result = match &self.images.scan {
            Some(report) if self.images.known(reference) => {
                Ok(ScanProgress::Report(report.clone()))
            }
            Some(_) => Err(no_such_image(reference)),
            None => Err(EngineError::Api("the scan failed".into())),
        };
        stream::once(ready(result)).boxed()
    }

    /// Removes nothing and reports nothing reclaimed.
    fn prune_build_cache(&self, _older_than: Duration) -> EngineFuture<u64> {
        ready(Ok(0)).boxed()
    }

    /// Reports the size of the record `id` in the disk use, unless a build uses it.
    fn prune_build_record(&self, id: &str, _older_than: Duration) -> EngineFuture<u64> {
        let reclaimed = self
            .disk
            .build_cache
            .iter()
            .find(|record| record.id == id && !record.in_use)
            .map_or(0, |record| record.size);
        ready(Ok(reclaimed)).boxed()
    }
}

impl FakeImages {
    fn known(&self, reference: &str) -> bool {
        self.images
            .iter()
            .any(|image| image.id == reference || image.repo_tags.iter().any(|t| t == reference))
    }
}

fn no_such_image(id: &str) -> EngineError {
    EngineError::Api(format!("No such image: {id}"))
}

#[cfg(test)]
mod tests;
