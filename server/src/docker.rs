use std::collections::HashMap;

use anyhow::anyhow;
use bollard::{
    container::{
        Config, CreateContainerOptions, ListContainersOptions, StartContainerOptions,
        WaitContainerOptions,
    },
    image::{self, CreateImageOptions},
    models::HostConfig,
    network::{ConnectNetworkOptions, CreateNetworkOptions, InspectNetworkOptions},
    service::{ContainerCreateResponse, CreateImageInfo, PortBinding},
    Docker,
};
use poem::{
    handler,
    web::{Data, Json},
};
use serde::Serialize;
use tokio::task::JoinHandle;
use url::Url;

use futures_util::stream::select;
use futures_util::stream::StreamExt;
use futures_util::stream::TryStreamExt;

/*

What information is needed for creating a new Docker-based app?

- Image name or URL
- App name (the same as the container name, also recorded in Caddy JSON config)
- Subdomain

*/

#[derive(Serialize)]
pub struct ContainerResponse {
    name: String,
    image_name: Option<String>,
    image_id: Option<String>,
}

#[handler]
pub async fn containers(docker: Data<&Docker>) -> anyhow::Result<Json<Vec<ContainerResponse>>> {
    let containers = docker
        .list_containers(Some(ListContainersOptions::<String> {
            all: true,
            ..Default::default()
        }))
        .await?;

    Ok(Json(
        containers
            .into_iter()
            .map(|container| ContainerResponse {
                name: container
                    .names
                    .unwrap_or(vec![])
                    .get(0)
                    .and_then(|name| name.strip_prefix("/"))
                    .unwrap_or("unnamed")
                    .to_string(),
                image_name: container.image,
                image_id: container.image_id.and_then(|id| {
                    id.split(':')
                        .collect::<Vec<&str>>()
                        .get(1)
                        .map(|id| id.to_string())
                }),
            })
            .collect(),
    ))
}

pub async fn initialize_network(docker: &Docker) -> anyhow::Result<()> {
    let network_exists = docker
        .inspect_network(
            "tower_network",
            Some(InspectNetworkOptions {
                verbose: false,
                scope: "local",
            }),
        )
        .await
        .is_ok();

    if !network_exists {
        docker
            .create_network(CreateNetworkOptions {
                name: "tower_network",
                ..Default::default()
            })
            .await?;
    }

    Ok(())
}

pub async fn restart_container(name: &String, docker: &Docker) -> anyhow::Result<()> {
    docker.restart_container(&name, None).await?;

    Ok(())
}

pub async fn create_container(
    container_name: &String,
    image_url: &String,
    exposing_port: &u16,
    docker: &Docker,
) -> anyhow::Result<ContainerCreateResponse> {
    let options = Some(CreateImageOptions {
        from_image: image_url.clone(),
        ..Default::default()
    });

    let _ = docker
        .create_image(options, None, None)
        .map(|item| {
            match &item {
                Ok(i) => println!("{:?}", i.progress),
                _ => (),
            }
            item
        })
        .try_collect::<Vec<CreateImageInfo>>()
        .await?;

    let container = docker
        .create_container(
            Some(CreateContainerOptions {
                name: container_name,
                platform: None,
            }),
            Config {
                image: Some(image_url.clone()),
                exposed_ports: {
                    let mut map: HashMap<String, HashMap<(), ()>> = HashMap::new();

                    map.insert(format!("{exposing_port}/tcp"), HashMap::new());

                    Some(map)
                },
                ..Default::default()
            },
        )
        .await?;

    docker
        .connect_network(
            "tower_network",
            ConnectNetworkOptions {
                container: container_name.clone(),
                ..Default::default()
            },
        )
        .await?;

    let _ = docker
        .start_container(&container_name, None::<StartContainerOptions<String>>)
        .await?;

    Ok(container)
}

pub async fn create_app(
    container_name: &String,
    image_url: &String,
    exposing_port: &u16,
    docker: &Docker,
) -> anyhow::Result<ContainerCreateResponse> {
    match docker.inspect_container(&container_name, None).await {
        Ok(_) => Err(anyhow!("This app already exists!").into()),
        Err(_) => create_container(container_name, image_url, exposing_port, docker).await,
    }
}

pub async fn delete_app(name: &String, docker: &Docker) -> anyhow::Result<()> {
    let docker_ = docker.clone();
    let name_ = name.clone();

    let _: JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {
        docker_.stop_container(&name_, None).await?;

        docker_
            .wait_container(
                &name_,
                Some::<WaitContainerOptions<String>>(WaitContainerOptions {
                    condition: "not-running".to_string(),
                }),
            )
            .try_collect::<Vec<_>>()
            .await?;

        docker_.remove_container(&name_, None).await?;

        Ok(())
    });

    Ok(())
}

pub async fn upgrade_to_latest(container_name: &String, docker: &Docker) -> anyhow::Result<()> {
    let container = docker.inspect_container(&container_name, None).await?;

    let container_image = container.config.clone().unwrap().image.unwrap();

    let _ = docker.stop_container(&container_name, None).await?;
    let _ = docker.remove_container(&container_name, None).await?;

    let _ = docker
        .create_image(
            Some(CreateImageOptions {
                from_image: container_image.clone(),
                ..Default::default()
            }),
            None,
            None,
        )
        .try_collect::<Vec<CreateImageInfo>>()
        .await?;

    dbg!(container);

    let _ = docker
        .create_container(
            Some(CreateContainerOptions {
                name: container_name,
                platform: None,
            }),
            Config {
                image: Some(container_image),
                ..Default::default()
            },
        )
        .await?;

    let _ = docker
        .start_container(&container_name, None::<StartContainerOptions<String>>)
        .await?;

    Ok(())
}
