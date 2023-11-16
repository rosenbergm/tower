use std::{
    fs::{self, DirEntry},
    path::Path,
};

use crate::{models::apps::App, mount::BASE};

pub fn read_configs_from_dir(type_: &str) -> Vec<App> {
    fs::read_dir(Path::new(BASE).join(type_))
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .filter_map(|entry| read_config_from_app(entry).ok())
        .collect()
}

fn read_config_from_app(dir: DirEntry) -> anyhow::Result<App> {
    let path = dir.path().join("config.toml");
    let config = fs::read_to_string(path)?;
    let model: App = toml::from_str(&config)?;

    Ok(model)
}
