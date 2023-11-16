use std::{ops::Deref, path::Path, time::Duration};

use anyhow::anyhow;
use bollard::{container::ListContainersOptions, service::Port, Docker};
use futures_util::{FutureExt, TryFutureExt};
use poem::{
    handler, head,
    http::HeaderMap,
    web::{Data, Form, Json},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::rc::Rc;
use tokio::task::JoinHandle;

const SECRET: &str = "2517d0a375522a501554a225e800ea7e51fb5117";

use crate::{
    caddy::{self, CaddyRoute, Handle, Match},
    docker,
    models::apps::{self, AppType},
    mount::{self, BASE},
    WSContainerData, WSMessage,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateDockerApp {
    pub name: String,
    pub image_url: String,
    pub exposing_port: u16,
    pub domain: String,
}

#[handler]
pub async fn create_docker_app(
    Form(config): Form<CreateDockerApp>,
    docker: Data<&Docker>,
    sender: Data<&tokio::sync::broadcast::Sender<WSMessage>>,
) -> anyhow::Result<()> {
    dbg!(&config);

    let docker_ = docker.clone();
    let sender_ = sender.clone();

    caddy::add_docker_app(&config.name, &config.domain, config.exposing_port).await?;

    let name = config.name.clone();

    sender.send(WSMessage {
        app_name: config.name.clone(),
        msg: WSContainerData::Creating,
    })?;

    // let sender_ = sender.clone();

    // tokio::spawn(async move {
    //     tokio::time::sleep(Duration::from_secs(10)).await;
    //     sender_.send(WSMessage {
    //         app_name: name.clone(),
    //         message: format!("Started creating container for app {name}"),
    //     });
    // });

    let _: JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {
        let _ = docker::create_app(
            &config.name,
            &config.image_url,
            &config.exposing_port,
            &docker_,
        )
        .await?;

        let container = docker_.inspect_container(&config.name, None).await?;

        sender_.send(WSMessage {
            app_name: config.name.clone(),
            msg: WSContainerData::Started,
        })?;

        Ok(())
    });

    Ok(())
}

#[handler]
pub async fn delete_docker_app(
    poem::web::Path(name): poem::web::Path<String>,
    docker: Data<&Docker>,
) -> anyhow::Result<()> {
    caddy::remove_app(&name).await?;

    // let app = apps::Entity::find()
    //     .filter(apps::Column::Name.eq(&name))
    //     .one(&db.conn)
    //     .await?;

    // match app {
    //     Some(a) => {
    //         apps::Entity::delete_by_id(a.id).exec(&db.conn).await?;
    //     }
    //     None => {}
    // }

    docker::delete_app(&name, &docker).await?;

    Ok(())
}

#[handler]
pub async fn restart_docker_app(
    poem::web::Path(name): poem::web::Path<String>,
    docker: Data<&Docker>,
    sender: Data<&tokio::sync::broadcast::Sender<WSMessage>>,
) -> anyhow::Result<()> {
    sender.send(WSMessage {
        app_name: name.clone(),
        msg: WSContainerData::Refreshing,
    })?;

    docker::restart_container(&name, &docker).await?;

    sender.send(WSMessage {
        app_name: name.clone(),
        msg: WSContainerData::Started,
    })?;

    Ok(())
}

#[handler]
pub async fn update_docker_app(
    poem::web::Path(name): poem::web::Path<String>,
    docker: Data<&Docker>,
    sender: Data<&tokio::sync::broadcast::Sender<WSMessage>>,
) -> anyhow::Result<()> {
    sender.send(WSMessage {
        app_name: name.clone(),
        msg: WSContainerData::Upgrading,
    })?;

    docker::upgrade_to_latest(&name, &docker).await?;

    sender.send(WSMessage {
        app_name: name.clone(),
        msg: WSContainerData::Started,
    })?;

    Ok(())
}

#[handler]
pub async fn update_docker_app_github(
    poem::web::Path(name): poem::web::Path<String>,
    headers: &HeaderMap,
    docker: Data<&Docker>,
    sender: Data<&tokio::sync::broadcast::Sender<WSMessage>>,
) -> anyhow::Result<()> {
    let signature = headers
        .get("x-hub-signature-256")
        .ok_or_else(|| anyhow!("FATAL! No secret in response"))?
        .to_str()?;

    let hashed_secret = format!("sha256={}", sha256::digest(SECRET));

    if signature != hashed_secret {
        return Err(anyhow!("Invalid secret").into());
    }

    sender.send(WSMessage {
        app_name: name.clone(),
        msg: WSContainerData::Upgrading,
    })?;

    docker::upgrade_to_latest(&name, &docker).await?;

    sender.send(WSMessage {
        app_name: name.clone(),
        msg: WSContainerData::Started,
    })?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateStaticApp {
    pub name: String,
    pub entrypoint: String,
    pub domain: String,
}

#[handler]
pub async fn create_static_app(Form(config): Form<CreateStaticApp>) -> anyhow::Result<()> {
    let mountpoint = mount::create_static_project(&config.name)?;

    let entrypoint = Path::new(&config.entrypoint);

    caddy::add_static_app(
        &config.name,
        &config.domain,
        mountpoint.as_path(),
        entrypoint,
    )
    .await?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DockerAppResponse {
    pub name: String,
    pub domain: String,
    pub container_details: ContainerDetails,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ContainerDetails {
    Creating,
    Running {
        image_url: Option<String>,
        exposing_port: Option<Vec<Port>>,
    },
    Error,
    Other,
}

#[handler]
pub async fn get_docker_apps(
    docker: Data<&Docker>,
) -> anyhow::Result<Json<Vec<DockerAppResponse>>> {
    let containers = docker
        .list_containers(Some(ListContainersOptions::<String> {
            all: true,
            ..Default::default()
        }))
        .await?;

    let routes = caddy::pull_config().await?;

    let apps = apps::all_docker_apps();

    let docker_apps: Vec<DockerAppResponse> =
        apps
        .into_iter()
        .filter_map(|app| {
            let container = containers.iter().find(|c| {
                c.names.as_ref()
                    .map(|names| names.iter().any(|n| n.contains(&app.name)))
                    .unwrap_or(false)
            });

            let route = routes.iter().find(|r| is_docker_app(r) && r.id.as_ref().map(|id| id.clone() == app.name).unwrap_or(false));

            match (container, route) {
                (Some(container), Some(route)) => {
                    dbg!("first");
                    let mut domain = None;

                    let route_ = route.clone();

                    let matches = Rc::new(route_.match_.as_ref().unwrap());
                    matches.iter().for_each(|m| match &m {
                        Match::Host(domains) => domains.get(0).clone_into(&mut domain),
                        _ => {},
                    });

                    dbg!(&container.state);

                    Some(DockerAppResponse {
                        name: app.name.clone(),
                        domain: domain.expect("Something just went terribly wrong. The route should have been associated with a domain.").to_string(),
                        container_details: container.state.as_ref().map(|state| {
                            match state.as_str() {
                                "created" => ContainerDetails::Creating,
                                "running" => ContainerDetails::Running { 
                                    image_url: container.image.clone()
                                        .and_then(|image| Path::new(&image).file_name()
                                        .and_then(|osstr| osstr.to_str())
                                        .map(|name| name.to_owned())),
                                    exposing_port: container.ports.clone(),
                                },
                                _ => ContainerDetails::Other
                            }
                        }).unwrap_or(ContainerDetails::Error)
                    })
                },
                (c, Some(route)) => {
                    dbg!("second");
                    let mut domain = None;

                    let route_ = route.clone();

                    let matches = Rc::new(route_.match_.as_ref().unwrap());
                    matches.iter().for_each(|m| match &m {
                        Match::Host(domains) => domains.get(0).clone_into(&mut domain),
                        _ => {},
                    });

                    Some(DockerAppResponse {
                        name: app.name.clone(),
                        domain: domain.expect("Something just went terribly wrong. The route should have been associated with a domain.").to_string(),
                        container_details: match c {
                            None => ContainerDetails::Creating,
                            Some(container) =>
                                container.state.as_ref().map(|state| {
                                    match state.as_str() {
                                        "created" => ContainerDetails::Creating,
                                        "running" => ContainerDetails::Running { 
                                            image_url: container.image.clone()
                                                .and_then(|image| Path::new(&image).file_name()
                                                .and_then(|osstr| osstr.to_str())
                                                .map(|name| name.to_owned())),
                                            exposing_port: container.ports.clone(),
                                        },
                                        _ => ContainerDetails::Other
                                    }
                                }).unwrap_or(ContainerDetails::Error)
                            ,
                        }
                    })
                }
                _ => {
                    dbg!(format!("App {0} is dead! Removing...", app.name));

                    // TODO: Store more info in the DB to be able to revive the caddy config and container.

                    None
                }
            }

        }).collect();

    Ok(Json(docker_apps))
}

fn is_docker_app(route: &CaddyRoute) -> bool {
    route
        .handle
        .as_ref()
        .map(|handles| {
            handles.into_iter().any(|handle| match handle {
                Handle::Subroute { routes: subroutes } => subroutes.into_iter().any(|subroute| {
                    subroute
                        .handle
                        .as_ref()
                        .map(|subhandles| {
                            subhandles.into_iter().any(|subhandle| match subhandle {
                                Handle::ReverseProxy { upstreams: _ } => true,
                                _ => false,
                            })
                        })
                        .unwrap_or(false)
                }),
                _ => false,
            })
        })
        .unwrap_or(false)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StaticAppResponse {
    pub name: String,
    pub domain: String,
    pub mountpoint: Option<String>,
    pub entrypoint: Option<String>,
}

#[handler]
pub async fn get_static_apps() -> anyhow::Result<Json<Vec<StaticAppResponse>>> {
    let routes = caddy::pull_config().await?;

    let apps = apps::all_static_apps();

    let static_apps: Vec<StaticAppResponse> =
        apps
        .into_iter()
        .filter_map(|app| {
            let route = routes.iter().find(|r| is_static_app(r) && r.id.as_ref().map(|id| id.clone() == app.name).unwrap_or(false));

            match route {
                Some(route) => {
                    let mut domain = None;

                    let route_ = route.clone();

                    let matches = Rc::new(route_.match_.as_ref().unwrap());
                    matches.iter().for_each(|m| match &m {
                        Match::Host(domains) => domains.get(0).clone_into(&mut domain),
                        _ => {},
                    });

                    let mut entrypoint = None;

                    let routes = Rc::new(route_.handle.as_ref().unwrap());
                    routes.iter().for_each(|r| match &r {
                        Handle::Subroute { routes: rs } => {
                            rs.iter().for_each(|r_| {
                                match &r_.match_.as_ref() {
                                    Some(r) => {
                                        let _ = r.iter().for_each(|m_| {
                                            match &m_ {
                                                Match::File { try_files: files } => {
                                                    files.last().clone_into(&mut entrypoint);
                                                },
                                                _ => {}
                                            }
                                        });
                                    },
                                    None => {}
                                }
                            }
                            )
                        },
                        _ => {}
                    });

                    Some(StaticAppResponse {
                        name: app.name.clone(),
                        domain: domain.expect("Something just went terribly wrong. The route should have been associated with a domain.").to_string(),
                        mountpoint: Path::new(BASE).join(app.name).to_str().map(|p| p.to_owned()),
                        entrypoint: entrypoint.clone().map(|e| e.to_owned())
                    })
                },
                None => {
                    None
                }
            }

        }).collect();

    Ok(Json(static_apps))
}

fn is_static_app(route: &CaddyRoute) -> bool {
    route
        .handle
        .as_ref()
        .map(|handles| {
            handles.into_iter().any(|handle| match handle {
                Handle::Subroute { routes: subroutes } => subroutes.into_iter().any(|subroute| {
                    subroute
                        .handle
                        .as_ref()
                        .map(|subhandles| {
                            subhandles.into_iter().any(|subhandle| match subhandle {
                                Handle::FileServer { hide: _ } => true,
                                _ => false,
                            })
                        })
                        .unwrap_or(false)
                }),
                _ => false,
            })
        })
        .unwrap_or(false)
}
