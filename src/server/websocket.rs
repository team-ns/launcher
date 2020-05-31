use std::sync::Arc;

use futures::{FutureExt, StreamExt};
use launcher_api::message::{
    AuthMessage, AuthResponse, ClientMessage, Error, ProfileResourcesMessage,
    ProfileResourcesResponse, ProfilesMessage, ServerMessage,
};
use log::error;
use rand::Rng;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::{mpsc, RwLock};
use warp::filters::ws::{Message, WebSocket};

use crate::LaunchServer;
use walkdir::WalkDir;

pub async fn ws_api(ws: WebSocket, server: Arc<RwLock<LaunchServer>>) {
    let (ws_tx, mut ws_rx) = ws.split();
    let (tx, rx) = mpsc::unbounded_channel();
    tokio::task::spawn(rx.forward(ws_tx).map(|result| {
        if let Err(e) = result {
            error!("Websocket send error: {}", e);
        }
    }));
    while let Some(result) = ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                error!("Websocket error");
                break;
            }
        };
        if let Ok(json) = msg.to_str() {
            if let Ok(message) = serde_json::from_str::<ClientMessage>(json) {
                match message {
                    ClientMessage::Auth(auth) => {
                        auth.handle(tx.clone(), server.clone()).await;
                    }
                    ClientMessage::Profiles(profiles) => {
                        profiles.handle(tx.clone(), server.clone()).await;
                    }
                    ClientMessage::ProfileResources(resources) => {
                        resources.handle(tx.clone(), server.clone()).await;
                    }
                }
            }
        }
    }
}

#[async_trait::async_trait]
pub trait Handle {
    async fn handle(
        &self,
        tx: UnboundedSender<Result<Message, warp::Error>>,
        server: Arc<RwLock<LaunchServer>>,
    );
}

#[async_trait::async_trait]
impl Handle for ProfileResourcesMessage {
    async fn handle(
        &self,
        tx: UnboundedSender<Result<Message, warp::Error>>,
        server: Arc<RwLock<LaunchServer>>,
    ) {
        let server = server.read().await;
        if server.config.profiles.contains(&self.profile) {
            let list = WalkDir::new(format!("static/{}", self.profile))
                .into_iter()
                .filter(|e| e.is_ok() && e.as_ref().ok().unwrap().metadata().unwrap().is_file())
                .map(|e| e.ok().unwrap().path().display().to_string())
                .collect::<Vec<String>>();
            let message = ServerMessage::ProfileResources(ProfileResourcesResponse { list });
            tx.send(Ok(Message::text(serde_json::to_string(&message).unwrap())));
        } else {
            let message = ServerMessage::Error(Error {
                msg: String::from("This profile doesn't exist!"),
            });
            tx.send(Ok(Message::text(serde_json::to_string(&message).unwrap())));
        }
    }
}

#[async_trait::async_trait]
impl Handle for ProfilesMessage {
    async fn handle(
        &self,
        tx: UnboundedSender<Result<Message, warp::Error>>,
        server: Arc<RwLock<LaunchServer>>,
    ) {
        unimplemented!()
    }
}

#[async_trait::async_trait]
impl Handle for AuthMessage {
    async fn handle(
        &self,
        tx: UnboundedSender<Result<Message, warp::Error>>,
        server: Arc<RwLock<LaunchServer>>,
    ) {
        let server = server.read().await;
        //TODO ADD IP FOR LIMITERS
        let ip = "".to_string();
        let password = server.security.decrypt(&self.password);
        let result = server.config.auth.auth(&self.login, &password, &ip).await;
        match result {
            Ok(result) => {
                if result.message.is_none() {
                    let digest = {
                        let mut rng = rand::thread_rng();
                        md5::compute(format!(
                            "{}{}{}",
                            rng.gen_range(1000000000, 2147483647),
                            rng.gen_range(1000000000, 2147483647),
                            rng.gen_range(0, 9)
                        ))
                    };
                    let access_token = format!("{:x}", digest);
                    let uuid = result.uuid.unwrap();
                    if server
                        .config
                        .auth
                        .update_access_token(&uuid, &access_token)
                        .await
                    {
                        tx.send(Ok(Message::text(
                            serde_json::to_string(&ServerMessage::Auth(AuthResponse {
                                uuid: uuid.to_string(),
                                access_token: access_token.to_string(),
                            }))
                            .unwrap(),
                        )));
                    }
                } else {
                    let message = ServerMessage::Error(Error {
                        msg: result.message.unwrap(),
                    });
                    tx.send(Ok(Message::text(serde_json::to_string(&message).unwrap())));
                }
            }
            Err(e) => {
                let message = ServerMessage::Error(Error { msg: e.msg });
                tx.send(Ok(Message::text(serde_json::to_string(&message).unwrap())));
            }
        }
    }
}
