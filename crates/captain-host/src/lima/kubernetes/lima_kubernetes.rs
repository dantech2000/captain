//! [`LimaKubernetes`]: the [`KubernetesHost`] of a [`LimaHost`].

use std::path::PathBuf;

use captain_core::kubernetes::{KubernetesHost, KubernetesSettings, KubernetesStatus, VersionList};
use captain_core::{HostError, HostFuture, HostStream};
use futures::StreamExt;
use futures::channel::mpsc;

use super::install;
use crate::LimaHost;
use crate::blocking::blocking;
use crate::cancel::Cancel;
use crate::k3s;

/// k3s in one Lima instance.
pub struct LimaKubernetes {
    host: LimaHost,
}

impl LimaKubernetes {
    pub fn new(host: LimaHost) -> Self {
        Self { host }
    }
}

impl KubernetesHost for LimaKubernetes {
    fn status(&self) -> HostFuture<KubernetesStatus> {
        let host = self.host.clone();
        blocking(move || Ok(host.kubernetes_status()))
    }

    fn versions(&self, refresh: bool) -> HostFuture<VersionList> {
        let paths = self.host.paths().clone();
        blocking(move || {
            Ok(k3s::list(
                &paths.k3s_versions_file(),
                &paths.k3s_cache(),
                refresh,
            ))
        })
    }

    fn enable(&self, settings: KubernetesSettings) -> HostStream<String> {
        let host = self.host.clone();
        on_thread(move |sink| {
            host.set_kubernetes_settings(settings.clone());
            host.with_running_engine(|limactl, paths| {
                install::install(limactl, paths, &settings, &Cancel::default(), sink).map(drop)
            })
        })
    }

    fn disable(&self) -> HostFuture<()> {
        let host = self.host.clone();
        blocking(move || {
            host.with_running_engine(|limactl, paths| {
                install::disable(limactl, paths, &Cancel::default())
            })
        })
    }

    fn reset(&self) -> HostStream<String> {
        let host = self.host.clone();
        on_thread(move |sink| {
            let settings = host.kubernetes_settings();
            host.with_running_engine(|limactl, paths| {
                sink("Deleting the Kubernetes cluster.".into());
                let cancel = Cancel::default();
                install::reset(limactl, paths, &cancel)?;
                if settings.enabled {
                    install::install(limactl, paths, &settings, &cancel, sink)?;
                }
                Ok(())
            })
        })
    }

    fn kubeconfig(&self) -> PathBuf {
        self.host.paths().kubeconfig()
    }
}

/// Runs `work` on a new thread and streams the lines it passes to its sink.
fn on_thread(
    work: impl FnOnce(&mut dyn FnMut(String)) -> Result<(), HostError> + Send + 'static,
) -> HostStream<String> {
    let (tx, rx) = mpsc::unbounded();
    std::thread::spawn(move || {
        let mut sink = |line: String| {
            tx.unbounded_send(Ok(line)).ok();
        };
        if let Err(error) = work(&mut sink) {
            tx.unbounded_send(Err(error)).ok();
        }
    });
    rx.boxed()
}
