use std::{collections::HashMap, fmt::format, path::Path};

use anyhow::anyhow;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

#[derive(Serialize, Deserialize, Debug)]
pub struct CaddyRoute {
    #[serde(rename = "@id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(rename = "match", skip_serializing_if = "Option::is_none")]
    pub match_: Option<Vec<Match>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<Vec<Handle>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminal: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Match {
    Host(Vec<String>),
    File { try_files: Vec<String> },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "handler", rename_all = "snake_case")]
pub enum Handle {
    StaticResponse {
        #[serde(skip_serializing_if = "Option::is_none")]
        headers: Option<HashMap<String, Vec<String>>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        status_code: Option<u16>,
        #[serde(skip_serializing_if = "Option::is_none")]
        body: Option<String>,
    },
    ReverseProxy {
        upstreams: Vec<Upstream>,
    },
    Vars {
        root: String,
    },
    Rewrite {
        uri: String,
    },
    Subroute {
        routes: Vec<CaddyRoute>,
    },
    FileServer {
        #[serde(skip_serializing_if = "Option::is_none")]
        hide: Option<Vec<String>>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
struct Route {
    routes: Vec<CaddyRoute>,

    #[serde(skip_serializing_if = "Option::is_none")]
    headers: Option<HashMap<String, Vec<String>>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    status_code: Option<u8>,

    #[serde(skip_serializing_if = "Option::is_none")]
    upstreams: Option<Vec<Upstream>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    root: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    uri: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Upstream {
    pub dial: String,
}

pub fn config_base() -> serde_json::Value {
    json!({
        "admin": {
            "listen": "0.0.0.0:2019"
        },
      "apps": {
        "http": {
          "servers": {
            "tower": {
              "listen": [
                // TODO: Change this to 443
                ":2009"
              ],
              "routes": []
            }
          }
        }
      }
    })
}

pub async fn initialize_tower() -> anyhow::Result<()> {
    let res = reqwest::get("http://localhost:2019/config/apps/http/servers/tower/routes").await?;

    let client = reqwest::Client::new();

    if res.status() == StatusCode::BAD_REQUEST {
        println!("Detected config that is now Tower, resetting...");
        client
            .post("http://localhost:2019/load")
            .json(&config_base())
            .send()
            .await?;
    }

    Ok(())
}

pub fn extract_routes(caddy_config: String) -> anyhow::Result<Vec<CaddyRoute>> {
    let mut value: Map<String, Value> = serde_json::from_str(&caddy_config)?;

    let apps = value.get_mut("apps").unwrap().as_object_mut().unwrap();
    let http = apps.get_mut("http").unwrap().as_object_mut().unwrap();
    let servers = http.get_mut("servers").unwrap().as_object_mut().unwrap();
    let tower_server = servers.get_mut("tower").unwrap().as_object_mut().unwrap();
    let routes_raw = tower_server.get("routes").unwrap();

    let routes: Vec<CaddyRoute> = serde_json::from_value(routes_raw.clone())?;

    Ok(routes)
}

pub async fn pull_config() -> anyhow::Result<Vec<CaddyRoute>> {
    let config = reqwest::get("http://localhost:2019/config/apps/http/servers/tower/routes")
        .await?
        .json()
        .await?;

    Ok(config)
}

pub async fn push_config(config: &Vec<CaddyRoute>) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let res = client
        .post("http://localhost:2019/config/apps/http/servers/tower/routes")
        .json(config)
        .send()
        .await?;

    if res.status() != StatusCode::OK {
        return Err(anyhow!("Could not push config!").into());
    }

    Ok(())
}

pub async fn remove_app(name: &String) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let res = client
        .delete(format!("http://localhost:2019/id/{name}"))
        .send()
        .await?;

    if res.status() != StatusCode::OK {
        return Err(anyhow!("Failed to remove app!").into());
    }

    Ok(())
}

pub async fn add_docker_app(name: &String, domain: &String, port: u16) -> anyhow::Result<()> {
    let route = CaddyRoute {
        id: Some(name.clone()),
        terminal: Some(true),
        match_: Some(vec![Match::Host(vec![domain.clone()])]),
        handle: Some(vec![Handle::Subroute {
            routes: vec![CaddyRoute {
                id: None,
                match_: None,
                handle: Some(vec![Handle::ReverseProxy {
                    upstreams: vec![Upstream {
                        dial: format!("{name}:{port}"),
                    }],
                }]),
                terminal: None,
            }],
        }]),
    };

    println!("{}", serde_json::to_string_pretty(&route)?);

    let client = reqwest::Client::new();
    let res = client
        .post(format!(
            "http://localhost:2019/config/apps/http/servers/tower/routes"
        ))
        .json(&route)
        .send()
        .await?;

    if res.status() != StatusCode::OK {
        return Err(anyhow!("Failed to add a Docker app!").into());
    }

    Ok(())
}

pub async fn add_static_app(
    name: &String,
    domain: &String,
    mountpoint: &Path,
    entrypoint: &Path,
) -> anyhow::Result<()> {
    let route = CaddyRoute {
        id: Some(name.clone()),
        terminal: Some(true),
        match_: Some(vec![Match::Host(vec![domain.clone()])]),
        handle: Some(vec![Handle::Subroute {
            routes: vec![
                CaddyRoute {
                    id: None,
                    match_: None,
                    handle: Some(vec![Handle::Vars {
                        root: mountpoint.to_str().unwrap().to_string(),
                    }]),
                    terminal: None,
                },
                CaddyRoute {
                    id: None,
                    match_: Some(vec![Match::File {
                        try_files: vec![
                            "{http.request.uri.path}".to_string(),
                            entrypoint.to_str().unwrap().to_string(),
                        ],
                    }]),
                    handle: Some(vec![Handle::Rewrite {
                        uri: "{http.matchers.file.relative}".to_string(),
                    }]),
                    terminal: None,
                },
                CaddyRoute {
                    id: None,
                    match_: None,
                    handle: Some(vec![Handle::FileServer { hide: None }]),
                    terminal: None,
                },
            ],
        }]),
    };

    let client = reqwest::Client::new();
    let res = client
        .post(format!(
            "http://localhost:2019/config/apps/http/servers/tower/routes"
        ))
        .json(&route)
        .send()
        .await?;

    if res.status() != StatusCode::OK {
        return Err(anyhow!("Failed to add a Static app!").into());
    }

    Ok(())
}
