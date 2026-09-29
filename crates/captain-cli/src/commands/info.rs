//! `captain info`: the resources for the next start, this computer, and the paths.

use std::path::PathBuf;

use anyhow::Result;
use captain_core::format::bytes_label;
use captain_host::LimaHost;
use serde::Serialize;

use crate::context::Context;
use crate::settings_keys::engine_resources;

#[derive(Serialize)]
struct Info {
    cpus: u32,
    memory_bytes: u64,
    disk_bytes: u64,
    host_cpus: u32,
    host_memory_bytes: u64,
    settings_file: PathBuf,
    endpoint: Option<String>,
    /// Captain Engine's Lima files; macOS only.
    lima: Option<Lima>,
}

#[derive(Serialize)]
struct Lima {
    lima_home: PathBuf,
    instance_dir: PathBuf,
    limactl: Option<PathBuf>,
}

pub fn run(context: &Context, json: bool) -> Result<()> {
    let settings = context.load_or_default();
    let resources = engine_resources(&settings, &context.machine);
    let info = Info {
        cpus: resources.cpus,
        memory_bytes: resources.memory_bytes,
        disk_bytes: resources.disk_bytes,
        host_cpus: context.machine.cpus,
        host_memory_bytes: context.machine.memory_bytes,
        settings_file: context.settings_path.clone(),
        endpoint: context.host(&settings).endpoint(),
        lima: cfg!(target_os = "macos").then(|| lima(&context.lima_host(&settings))),
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&info)?);
        return Ok(());
    }
    println!("Resources:       {}", resources.summary());
    println!(
        "This computer:   {} CPUs · {} memory",
        info.host_cpus,
        bytes_label(info.host_memory_bytes)
    );
    println!("Settings file:   {}", info.settings_file.display());
    println!(
        "Endpoint:        {}",
        info.endpoint.as_deref().unwrap_or("-")
    );
    if let Some(lima) = &info.lima {
        let limactl = lima.limactl.as_ref().map(|path| path.display().to_string());
        println!("LIMA_HOME:       {}", lima.lima_home.display());
        println!("Instance:        {}", lima.instance_dir.display());
        println!(
            "limactl:         {}",
            limactl.as_deref().unwrap_or("not found")
        );
    }
    Ok(())
}

fn lima(host: &LimaHost) -> Lima {
    let paths = host.paths();
    Lima {
        lima_home: paths.lima_home.clone(),
        instance_dir: paths.instance_dir(),
        limactl: host.limactl_path().ok(),
    }
}
