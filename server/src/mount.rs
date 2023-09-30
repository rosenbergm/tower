pub const BASE: &str = "/home/martin/tower/server/test_base";

use std::collections::HashMap;
use std::fs;
use std::path;
use std::path::Path;
use std::path::PathBuf;
use std::vec;

use anyhow::anyhow;
use poem::error::InternalServerError;

use crate::caddy;
use crate::caddy::CaddyRoute;
use crate::caddy::Handle;
use crate::caddy::Match;
use crate::caddy::Upstream;

pub fn initialize_fs() -> anyhow::Result<()> {
    fs::create_dir_all(Path::new(BASE))
        .map_err(|_| anyhow!("Failed to create Tower filesystem!").into())
}

pub fn create_project_directory(path: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(path).map_err(|_| anyhow!("Failed to create project directory!").into())
}

pub fn create_static_project(project_name: &String) -> anyhow::Result<PathBuf> {
    // 1. Create project directory
    // 2. Upload all necessary files
    // 3. Update caddyserver config

    let project_path = Path::new(BASE).join(&project_name);

    if project_path.is_dir() {
        return Err(anyhow!("Project already exists!").into());
    }

    create_project_directory(&project_path)?;

    // TODO: Upload all files to directory

    Ok(project_path)
}
