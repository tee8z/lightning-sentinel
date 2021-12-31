use super::client_wrapper::{build_url, ClientWrapper};
use crate::channels::{ChannelMessage, ChannelType};
use crate::config_wrapper::Settings;
use crate::objects::LnGetInfo;
use crate::pickle_jar::{PickleJar, Row};
use anyhow::Result;
use log::{error, info};
use reqwest::{
    header::{HeaderMap, HeaderValue},
    StatusCode,
};
use serde::{Deserialize, Serialize};
use serde_json;
use std::{fmt, sync::Arc};
use tokio::sync::mpsc::Sender;
use tokio::time::{interval, Duration};

#[derive(Serialize, Deserialize)]
struct UserInfoLn {
    pub tor_api_url: String,
    pub num_pending_channels: i64,
    pub num_active_channels: i64,
    pub num_inactive_channels: i64,
    pub num_peers: i64,
}

impl fmt::Display for UserInfoLn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return write!(
            f,
            r#"(
            'tor_api_url': '{}',
            'num_pending_channels': '{}',
            'num_active_channels': '{}'
            'num_inactive_channels': '{}'
            'num_peers': '{}'
        )"#,
            self.tor_api_url,
            self.num_pending_channels,
            self.num_active_channels,
            self.num_inactive_channels,
            self.num_peers
        );
    }
}

pub fn setup_client(settings: &Settings) -> ClientWrapper {
    ClientWrapper::new(settings)
}

// Polling lightning node done here
pub async fn check_hidden_service(
    client: &ClientWrapper,
    ln_info: ChannelMessage,
    pickle: PickleJar,
    send_tel: Sender<ChannelMessage>,
) {
    let command = "/v1/getinfo";

    let resolved_data =
        handle_check_service(ln_info.clone(), PickleJar::new(Arc::clone(&pickle.db))).await;
    let mut interval = interval(Duration::from_secs(20));
    info!(
        "(check_hidden_service): ln_info.command {}",
        ln_info.command.clone()
    );
    let mut send_status = !ln_info.command.is_empty();
    loop {
        let pickle_get = PickleJar::new(Arc::clone(&pickle.db));
        let pickle_remove = PickleJar::new(Arc::clone(&pickle.db));
        let row = pickle_get.get(&ln_info.clone().chat_id).await;

        //Stops watching this node
        if !row.is_watching {
            pickle_remove.remove(&ln_info.clone().chat_id).await;
            break;
        }

        interval.tick().await;
        let url = &resolved_data.0;
        let macaroon = &resolved_data.1;
        match get_command_node(
            client,
            ln_info.clone(),
            url.to_string(),
            macaroon.to_string(),
            send_tel.clone(),
            command.to_string(),
            send_status,
        )
        .await
        {
            Ok(_) => {
                info!("(check_hidden_service): get_command_node OK");
            }
            Err(e) => {
                error!("(check_hidden_service): {}", e);
            }
        }
        send_status = false;
        info!("(check_hidden_service) Command: {}", ln_info.command);
        if ln_info.command == "status" {
            break;
        }
    }
}

pub async fn handle_check_service(ln_info: ChannelMessage, pickle: PickleJar) -> (String, String) {
    let mut row = PickleJar::new(Arc::clone(&pickle.db))
        .get(&ln_info.chat_id)
        .await;

    if row.node_url.is_empty() {
        row = Row {
            telegram_chat_id: ln_info.chat_id,
            node_url: ln_info.node_url.clone(),
            is_watching: true,
            macaroon: ln_info.macaroon.clone(),
        };
        PickleJar::new(Arc::clone(&pickle.db))
            .set(&ln_info.chat_id.to_string(), row.clone())
            .await;
    }

    (row.node_url, row.macaroon)
}

//TODO: Clean response from node to be clear/simple to end user
pub async fn get_command_node(
    client: &ClientWrapper,
    ln_info: ChannelMessage,
    check_url: String,
    macaroon: String,
    send_tel: Sender<ChannelMessage>,
    command: String,
    send_status: bool,
) -> Result<(), reqwest::Error> {
    let url = build_url(check_url.clone(), &command);
    let headers = build_headers(&macaroon);
    let res = client.client.get(url).headers(headers).send().await?;

    info!("(get_command_node) Status: {}", res.status());

    match res.status() {
        StatusCode::OK => {
            if !send_status {
                return Ok(());
            }

            handle_success_request(check_url.clone(), res, ln_info, &command, send_tel.clone())
                .await
                .unwrap();
        }
        _ => {
            handle_request_err(check_url.clone(), res, ln_info, &command, send_tel.clone())
                .await
                .unwrap();
        }
    }
    Ok(())
}

fn build_headers(macaroon: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let header_val = HeaderValue::from_str(macaroon).unwrap();
    headers.insert("Grpc-Metadata-macaroon", header_val);
    let header_val = HeaderValue::from_str("application/json").unwrap();
    headers.insert("Content-Type", header_val);
    headers
}

async fn handle_success_request(
    tor_api_url: String,
    res: reqwest::Response,
    ln_info: ChannelMessage,
    command: &str,
    send_tel: Sender<ChannelMessage>,
) -> Result<(), reqwest::Error> {
    let message_text;
    match res.text().await {
        Ok(text) => {
            message_text = text;
        }
        Err(error) => {
            message_text = error.to_string();
        }
    };
    let to_send;

    match serde_json::from_str::<LnGetInfo>(&message_text) {
        Ok(ln_response) => {
            to_send = UserInfoLn {
                tor_api_url,
                num_pending_channels: ln_response.num_pending_channels,
                num_active_channels: ln_response.num_active_channels,
                num_inactive_channels: ln_response.num_inactive_channels,
                num_peers: ln_response.num_peers,
            }
            .to_string();
        }
        Err(error) => {
            to_send = format!("tor_api_url: {}\nerror: {}", tor_api_url, error.to_string());
        }
    }

    let tel_message = ChannelMessage {
        channel_type: ChannelType::Tel,
        command: command.to_string(),
        message: to_send,
        node_url: "".to_string(),
        chat_id: ln_info.chat_id,
        macaroon: "".to_string(),
    };

    info!("(handle_success_request) tel_message: {}", tel_message);
    if let Err(e) = send_tel.send(tel_message).await {
        error!("(handle_success_request) channel send error: {0}", e);
    }
    Ok(())
}

async fn handle_request_err(
    tor_api_url: String,
    res: reqwest::Response,
    ln_info: ChannelMessage,
    command: &str,
    send_tel: Sender<ChannelMessage>,
) -> Result<(), reqwest::Error> {
    let message_text;
    match res.text().await {
        Ok(text) => {
            message_text = format!("tor_api_url: {}\nerror: {}", tor_api_url, text);
        }
        Err(error) => {
            message_text = format!("tor_api_url: {}\nerror: {}", tor_api_url, error.to_string());
        }
    };

    let tel_message = ChannelMessage {
        channel_type: ChannelType::Tel,
        command: command.to_string(),
        message: message_text,
        node_url: "".to_string(),
        chat_id: ln_info.chat_id,
        macaroon: "".to_string(),
    };

    info!("(handle_request_err) tel_message: {}", tel_message);
    if let Err(e) = send_tel.send(tel_message).await {
        error!("(handle_request_err) channel send error {0}", e);
    }
    Ok(())
}
