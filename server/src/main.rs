use anyhow::Result;
use bollard::errors;
use bollard::service::Port;
use bollard::{container::ListContainersOptions, image::ListImagesOptions, Docker};
use dotenv::dotenv;
use futures_util::{SinkExt, StreamExt};
use poem::middleware::Cors;
use poem::web::Data;
use poem::{delete, post, IntoResponse};
use poem::{
    get, handler,
    listener::TcpListener,
    web::{
        websocket::{Message, WebSocket},
        Json, Path,
    },
    EndpointExt, Route, Server,
};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use url::Url;

use std::env;

mod apps;
mod caddy;
mod docker;
mod fsdb;
mod mount;

mod models;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum WSContainerData {
    Creating,
    Error,
    Refreshing,
    Started,
    Upgrading,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WSMessage {
    pub app_name: String,
    pub msg: WSContainerData,
}

impl ToString for WSMessage {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

#[handler]
fn ws(
    Path(app_name): Path<String>,
    ws: WebSocket,
    sender: Data<&tokio::sync::broadcast::Sender<WSMessage>>,
) -> impl IntoResponse {
    let sender = sender.clone();
    let mut receiver = sender.subscribe();
    ws.on_upgrade(move |socket| async move {
        let (mut sink, mut stream) = socket.split();

        tokio::spawn(async move {
            while let Some(Ok(msg)) = stream.next().await {
                if let Message::Text(text) = msg {
                    break;
                    // if sender.send(format!("{app_name}: {text}")).is_err() {
                    //     break;
                    // }
                }
            }
        });

        tokio::spawn(async move {
            while let Ok(msg) = receiver.recv().await {
                if msg.app_name == app_name {
                    if sink.send(Message::Text(msg.to_string())).await.is_err() {
                        break;
                    }
                }
            }
        });
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv()?;

    let docker = Docker::connect_with_socket_defaults()?;

    docker::initialize_network(&docker).await?;

    mount::initialize_fs()?;

    caddy::initialize_tower().await?;

    let channel = tokio::sync::broadcast::channel::<WSMessage>(32);

    let cors = Cors::new()
        .allow_origin("http://localhost:5173")
        .allow_origin("http://127.0.0.1:5173");

    let app = Route::new()
        .at("/ahoj", get(docker::containers))
        .nest(
            "/docker",
            Route::new()
                .at("/apps", get(apps::get_docker_apps))
                .at("/new", post(apps::create_docker_app))
                .at("/:name", delete(apps::delete_docker_app))
                .at("/:name/restart", post(apps::restart_docker_app))
                .at("/:name/update/manual", post(apps::update_docker_app))
                .at("/:name/update/github", post(apps::update_docker_app_github))
                .data(channel.0.clone()),
        )
        .nest(
            "/static",
            Route::new()
                .at("/apps", get(apps::get_static_apps))
                .at("/new", post(apps::create_static_app)),
        )
        .at("/ws/:app_name", get(ws.data(channel.0.clone())))
        .data(docker)
        .with(cors);

    Server::new(TcpListener::bind("127.0.0.1:8000"))
        .run(app)
        .await?;

    Ok(())
}
