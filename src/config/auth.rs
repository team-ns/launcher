use futures::TryFutureExt;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use reqwest::Client;
use async_trait::async_trait;

use crate::config::{JsonAuthProvider, None};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub access_token: Option<String>,
    pub server_id: Option<String>,
    pub uuid: Uuid,
    pub username: String,
}

#[derive(Deserialize, Serialize)]
pub struct AuthResult {
    pub uuid: Option<Uuid>,
    pub message: Option<String>,
}

pub struct Error {
    pub message: String,
}

#[async_trait(? Send)]
pub trait AuthProvide {
    async fn auth(
        &self,
        login: &String,
        password: &String,
        ip: &String,
    ) -> Result<AuthResult, Error>;
    async fn get_entry(&self, uuid: &Uuid) -> Result<Entry, Error>;
    async fn get_entry_from_name(&self, username: &String) -> Result<Entry, Error>;
    async fn update_access_token(&self, uuid: &Uuid, token: &String);
    async fn update_server_id(&self, uuid: &Uuid, server_id: &String);
}

#[async_trait(? Send)]
impl AuthProvide for JsonAuthProvider {
    async fn auth(
        &self,
        login: &String,
        password: &String,
        ip: &String,
    ) -> Result<AuthResult, Error> {
        let client = Client::default();

        let result = client
            .post(&self.auth_url)
            .json(&serde_json::json!({
                "username": login,
                "password": password,
                "ip": ip
            }))
            .send()
            .await
            .map_err(|_e| Error {
                message: "Can't connect".to_string(),
            })?
            .json()
            .map_err(|_e| Error {
                message: "Can't parse json".to_string(),
            })
            .await?;
        Ok(result)
    }

    async fn get_entry(&self, uuid: &Uuid) -> Result<Entry, Error> {
        let client = Client::default();
        Ok(client
            .post(&self.entry_url)
            .json(&serde_json::json!({ "uuid": uuid }))
            .send()
            .await
            .map_err(|_e| Error {
                message: "Can't connect".to_string(),
            })?
            .json()
            .map_err(|_e| Error {
                message: "Can't parse json".to_string(),
            })
            .await?)
    }

    async fn get_entry_from_name(&self, username: &String) -> Result<Entry, Error> {
        let client = Client::default();
        Ok(client
            .post(&self.entry_url)
            .json(&serde_json::json!({ "username": username }))
            .send()
            .await
            .map_err(|_e| Error {
                message: "Can't connect".to_string(),
            })?
            .json()
            .map_err(|_e| Error {
                message: "Can't parse json".to_string(),
            })
            .await?)
    }

    async fn update_access_token(&self, uuid: &Uuid, token: &String) {
        let client = Client::default();
        let response = client
            .post(&self.update_access_token_url)
            .json(&serde_json::json!({
                "uuid": uuid,
                "accessToken": token
            }))
            .send()
            .await;
    }

    async fn update_server_id(&self, uuid: &Uuid, server_id: &String) {
        let client = Client::default();
        let response = client
            .post(&self.update_server_id_url)
            .json(&serde_json::json!({
            "uuid": uuid,
            "serverId": server_id
            }))
            .send()
            .await;
    }
}

#[async_trait(? Send)]
impl AuthProvide for None {
    async fn auth(
        &self,
        _login: &String,
        _password: &String,
        _ip: &String,
    ) -> Result<AuthResult, Error> {
        unimplemented!()
    }

    async fn get_entry(&self, _uuid: &Uuid) -> Result<Entry, Error> {
        unimplemented!()
    }

    async fn get_entry_from_name(&self, _username: &String) -> Result<Entry, Error> {
        unimplemented!()
    }

    async fn update_access_token(&self, _uuid: &Uuid, _token: &String) {
        unimplemented!()
    }

    async fn update_server_id(&self, _uuid: &Uuid, _server_id: &String) {
        unimplemented!()
    }
}
