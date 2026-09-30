//! `captain list-settings`: the settings that `set` changes, their values, and
//! whether the settings file sets them or they have their defaults.

use anyhow::Result;
use captain_core::settings::{LoadedSettings, Settings};
use clap::ValueEnum;
use serde_json::{Map, Value, json};

use crate::context::Context;
use crate::settings_keys::SettingKey;

pub fn run(context: &Context, json: bool) -> Result<()> {
    let loaded = Settings::read(&context.settings_path).unwrap_or_else(|error| {
        eprintln!("captain: {error:#}; using the defaults");
        LoadedSettings {
            settings: Settings::default(),
            problems: Vec::new(),
            text: String::new(),
        }
    });
    for problem in &loaded.problems {
        eprintln!("captain: {problem}");
    }
    let rows = SettingKey::value_variants().iter().map(|key| {
        let value = key.value(&loaded.settings, &context.machine);
        let source = if loaded.sets(key.file_key()) {
            "file"
        } else {
            "default"
        };
        (key.name(), value, source)
    });
    if json {
        let map: Map<String, Value> = rows
            .map(|(key, value, source)| (key, json!({"value": value, "source": source})))
            .collect();
        println!("{}", serde_json::to_string_pretty(&map)?);
    } else {
        for (key, value, source) in rows {
            println!("{key:<21} {value:<10} {source}");
        }
    }
    Ok(())
}
