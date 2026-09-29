//! What every command needs: the settings file, this computer, and Captain Engine,
//! built the same way the app builds them at launch.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::{Context as _, Result, bail};
use captain_core::EngineHost;
use captain_core::process_lock::{
    CLI_RESTORE_NOTE, ProcessLock, app_lock_path, settings_lock_path,
};
use captain_core::settings::Settings;
use captain_host::{LimaHost, LimaPaths, machine};

use crate::settings_keys::{Machine, engine_resources};

pub struct Context {
    /// `--settings`, `CAPTAIN_SETTINGS`, or the app's own file (ADR 0004).
    pub settings_path: PathBuf,
    pub machine: Machine,
    /// `--lima-home` and `--instance`: another instance, for tests.
    pub engine_paths: Option<LimaPaths>,
    /// True while this command holds `app.lock` itself. Windows cannot read the
    /// note of a held lock, so the note alone cannot tell.
    pub(crate) holds_app: AtomicBool,
}

impl Context {
    pub fn new(
        settings: Option<PathBuf>,
        lima_home: Option<PathBuf>,
        instance: Option<String>,
    ) -> Result<Self> {
        let settings_path = match settings {
            Some(path) => path,
            None => dirs::config_dir()
                .context("cannot find the config folder")?
                .join("Captain")
                .join("settings.json"),
        };
        Ok(Self {
            settings_path,
            machine: Machine {
                cpus: machine::host_cpus(),
                memory_bytes: machine::host_memory(),
                captain_available: captain_host::captain_engine_available(),
            },
            engine_paths: lima_home.map(|lima_home| LimaPaths {
                lima_home,
                instance: instance.unwrap_or_else(|| captain_host::INSTANCE.into()),
            }),
            holds_app: AtomicBool::new(false),
        })
    }

    /// The saved settings. A file that is not valid JSON fails, so `set` never
    /// replaces a file the user can still fix.
    pub fn load(&self) -> Result<Settings> {
        Ok(Settings::load(&self.settings_path)?)
    }

    /// The saved settings, or the defaults with a warning, like the app at launch.
    pub fn load_or_default(&self) -> Settings {
        self.load().unwrap_or_else(|error| {
            eprintln!("captain: {error:#}; using the defaults");
            Settings::default()
        })
    }

    /// Loads the settings, changes them with `change`, and saves them, all under the
    /// settings lock, so two commands never lose each other's change. It refuses
    /// with `refusal` while the app runs, because the app writes the whole file on
    /// each change. The check is inside the lock, so an app that starts meanwhile
    /// reads the saved file.
    pub fn update_settings<T>(
        &self,
        refusal: &str,
        change: impl FnOnce(&mut Settings) -> Result<T>,
    ) -> Result<T> {
        let path = settings_lock_path(&self.settings_path);
        let _lock = ProcessLock::acquire(&path)
            .with_context(|| format!("cannot lock {}", path.display()))?;
        if self.app_running() {
            bail!("{refusal}");
        }
        let mut settings = self.load()?;
        let result = change(&mut settings)?;
        settings.save(&self.settings_path)?;
        Ok(result)
    }

    /// True while the Captain app runs with this settings file, or when that
    /// cannot be checked. A restore's own hold on `app.lock` does not count.
    pub fn app_running(&self) -> bool {
        if self.holds_app.load(Ordering::SeqCst) {
            return false;
        }
        ProcessLock::holder(&app_lock_path(&self.settings_path))
            .is_some_and(|note| note != CLI_RESTORE_NOTE)
    }

    /// Holds `app.lock`, so the app cannot start until the lock drops. Fails with
    /// `refusal` while the app runs.
    pub fn exclude_app(&self, refusal: &str) -> Result<AppHold<'_>> {
        let path = app_lock_path(&self.settings_path);
        match ProcessLock::try_acquire(&path, CLI_RESTORE_NOTE) {
            Ok(Some(lock)) => {
                self.holds_app.store(true, Ordering::SeqCst);
                Ok(AppHold {
                    _lock: lock,
                    held: &self.holds_app,
                })
            }
            Ok(None) => bail!("{refusal}"),
            Err(error) => Err(error).with_context(|| format!("cannot lock {}", path.display())),
        }
    }

    /// Captain Engine with the saved resources and Docker daemon settings.
    pub fn host(&self, settings: &Settings) -> Arc<dyn EngineHost> {
        let host = match &self.engine_paths {
            Some(_) => Arc::new(self.lima_host(settings)),
            None => captain_host::default_host(engine_resources(settings, &self.machine)),
        };
        host.set_daemon(settings.engine_daemon.clone());
        host.set_kubernetes(settings.kubernetes.clone());
        host
    }

    /// Captain Engine's Lima VM, or the one `--lima-home` names, with the saved
    /// resources.
    pub fn lima_host(&self, settings: &Settings) -> LimaHost {
        let resources = engine_resources(settings, &self.machine);
        match &self.engine_paths {
            Some(paths) => LimaHost::with_paths(paths.clone(), resources),
            None => LimaHost::new(resources),
        }
    }
}

/// `app.lock` while this command holds it. Dropping it releases the lock.
pub struct AppHold<'a> {
    _lock: ProcessLock,
    held: &'a AtomicBool,
}

impl Drop for AppHold<'_> {
    fn drop(&mut self) {
        self.held.store(false, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests;
