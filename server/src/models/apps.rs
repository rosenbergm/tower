use std::{
    fs::{self, DirEntry},
    path::Path,
};

use serde::{Deserialize, Serialize};

use crate::{fsdb::read_configs_from_dir, mount::BASE};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AppType {
    Docker,
    Static,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct App {
    pub name: String,

    #[serde(rename = "type")]
    pub type_: AppType,
}

pub fn all_static_apps() -> Vec<App> {
    read_configs_from_dir("static")
}

pub fn all_docker_apps() -> Vec<App> {
    read_configs_from_dir("docker")
}
