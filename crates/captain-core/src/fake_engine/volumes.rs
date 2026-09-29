use std::collections::BTreeMap;

use futures::FutureExt;
use futures::future::ready;

use super::FakeEngine;
use crate::model::{Volume, VolumePrune, VolumeUser};
use crate::store::{prunable_volumes, validate_name};
use crate::{EngineError, EngineFuture, VolumeApi};

/// The fake engine's volumes data. Create, remove, and prune check their input the
/// way the engine does, but do not change the list.
#[derive(Debug, Clone, Default)]
pub struct FakeVolumes {
    pub volumes: Vec<Volume>,
    /// The containers that use each volume, by volume name.
    pub users: BTreeMap<String, Vec<VolumeUser>>,
}

impl VolumeApi for FakeEngine {
    fn list_volumes(&self) -> EngineFuture<Vec<Volume>> {
        ready(Ok(self.volumes.volumes.clone())).boxed()
    }

    fn create_volume(&self, name: &str) -> EngineFuture<()> {
        let result = match validate_name(name) {
            Err(error) => Err(EngineError::Api(error.to_string())),
            Ok(()) if self.volumes.volumes.iter().any(|v| v.name == name) => {
                Err(EngineError::Api(format!("volume {name} already exists")))
            }
            Ok(()) => Ok(()),
        };
        ready(result).boxed()
    }

    fn remove_volume(&self, name: &str) -> EngineFuture<()> {
        let result = match self.volumes.volumes.iter().find(|v| v.name == name) {
            None => Err(EngineError::Api(format!("no such volume: {name}"))),
            Some(volume) if volume.is_in_use() => {
                Err(EngineError::Api(format!("volume {name} is in use")))
            }
            Some(_) => Ok(()),
        };
        ready(result).boxed()
    }

    fn volume_users(&self, name: &str) -> EngineFuture<Vec<VolumeUser>> {
        let users = self.volumes.users.get(name).cloned().unwrap_or_default();
        ready(Ok(users)).boxed()
    }

    /// Reports what the engine would remove. Sizes that are unknown count as zero.
    fn prune_unused_volumes(&self, all: bool, label: Option<&str>) -> EngineFuture<VolumePrune> {
        let pruned = prunable_volumes(&self.volumes.volumes, all, label);
        let report = VolumePrune {
            removed: pruned.iter().map(|v| v.name.clone()).collect(),
            reclaimed_bytes: pruned.iter().filter_map(|v| v.size_bytes).sum(),
        };
        ready(Ok(report)).boxed()
    }
}
