//! `captain list-settings`: the settings that `set` changes, and their values.

use anyhow::Result;
use clap::ValueEnum;
use serde_json::{Map, Value};

use crate::context::Context;
use crate::settings_keys::SettingKey;

pub fn run(context: &Context, json: bool) -> Result<()> {
    let settings = context.load_or_default();
    let values = SettingKey::value_variants()
        .iter()
        .map(|key| (key.name(), key.value(&settings, &context.machine)));
    if json {
        let map: Map<String, Value> = values.map(|(k, v)| (k, Value::String(v))).collect();
        println!("{}", serde_json::to_string_pretty(&map)?);
    } else {
        for (key, value) in values {
            println!("{key:<21} {value}");
        }
    }
    Ok(())
}
