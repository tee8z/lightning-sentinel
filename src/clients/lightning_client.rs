use tokio::time::{Duration, interval};
use crate::channels::{ChannelMessage, ChannelType};
use crate::config_wrapper::Settings;
use crate::pickle_jar::{PickleJar,Row};
use crate::objects::LnGetInfo;
use super::client_wrapper::{ClientWrapper, build_url};
use tokio::sync::mpsc::{Sender};
use reqwest::{
    StatusCode,
    header::{HeaderMap, HeaderValue}
};
use anyhow::Result;
use log::{info,error};
use serde_json;
use serde::{Serialize, Deserialize};
use std::{
    sync::Arc,
    fmt};


#[derive(Serialize, Deserialize)]
struct UserInfoLn {
    pub commit_hash: String, 
    pub identity_pubkey: String, 
    pub alias: String, 
    pub num_pending_channels: i64, 
    pub num_active_channels: i64, 
    pub num_inactive_channels: i64, 
    pub num_peers: i64, 
}

impl fmt::Display for UserInfoLn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return write!(f, r#"(
            'commit_hash': '{}',
            'identity_pubkey': '{}',
            'alias': '{}',
            'num_pending_channels': '{}',
            'num_active_channels': '{}'
            'num_inactive_channels': '{}'
            'num_peers': '{}'
        )"#, self.commit_hash, self.identity_pubkey, self.alias, self.num_pending_channels, self.num_active_channels, self.num_inactive_channels, self.num_peers);
    }
}

pub fn setup_client(settings: &Settings) -> ClientWrapper {
    let client = ClientWrapper::new(settings);
    return client;
}

// Polling lightning node done here
pub async fn check_hidden_service(client: &ClientWrapper, ln_info: ChannelMessage, pickle: PickleJar, send_tel:Sender<ChannelMessage>) {
    let command = "/v1/getinfo";

    let resolved_data = handle_check_service(ln_info.clone(),  PickleJar::new(Arc::clone(&pickle.db))).await;
    let mut interval = interval(Duration::from_secs(20));
    info!("{}", ln_info.command.clone());
    let mut send_status = if ln_info.command.len() == 0  { false } else { true };
    loop {
        let pickle_get = PickleJar::new(Arc::clone(&pickle.db));
        let pickle_remove = PickleJar::new(Arc::clone(&pickle.db));
        let row = pickle_get.get(&ln_info
            .clone()
            .chat_id)
            .await;

        //Stops watching this node
        if row.is_watching == false {
            pickle_remove.remove(&ln_info
                .clone()
                .chat_id)
                .await;
            break;
        }

        interval.tick().await;
        let url = &resolved_data.0;
        let macaroon = &resolved_data.1;
        match get_command_node(&client, ln_info.clone(), url.to_string(), macaroon.to_string(), send_tel.clone(), command.to_string(), send_status)
            .await {
                Ok(_) => { 
                    info!("OK"); 
                }
                Err(e) => { error!("{}", e); }
            }
        send_status = false;
        info!("Command: {}",ln_info.command);
        if ln_info.command == "status" {
            break;
        }
    }

}

pub async fn handle_check_service(ln_info: ChannelMessage, pickle: PickleJar) -> (String, String) {

    let mut row =  PickleJar::new(Arc::clone(&pickle.db))
                        .get(&ln_info.chat_id)
                        .await;

    if row.node_url.len() == 0 {
        row = Row{
            telegram_chat_id: ln_info.chat_id.clone(),
            node_url: ln_info.node_url.clone(),
            is_watching: true,
            macaroon: ln_info.macaroon.clone(),
        };
        PickleJar::new(Arc::clone(&pickle.db))
                .set(&ln_info.chat_id.to_string(), row.clone())
                .await;
    }

    return (row.node_url, row.macaroon);   
    
}


//TODO: Clean response from node to be clear/simple to end user
pub async fn get_command_node(client: &ClientWrapper, ln_info: ChannelMessage, check_url:String, macaroon:String, send_tel:Sender<ChannelMessage>, command:String, send_status:bool)-> Result<(), reqwest::Error> {
    
    let url = build_url(check_url, &command);
    let headers = build_headers(&macaroon);
    let res = client
                .client
                .get(url)
                .headers(headers)
                .send()
                .await?;

    info!("(get_command_node) Status: {}", res.status());

    match res.status() {
        StatusCode::OK => {
            if !send_status {
                return Ok(())
            }
            
            handle_success_request(res,ln_info, &command, send_tel.clone())
                    .await
                    .unwrap();
            
        },
        _ => { handle_request_err(res,ln_info, &command, send_tel.clone())
                        .await
                        .unwrap();
        }
    }
    Ok(())
}

fn build_headers(macaroon: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let header_val = HeaderValue::from_str(macaroon).unwrap();
    headers.insert("Grpc-Metadata-macaroon",header_val);
    let header_val = HeaderValue::from_str(&"application/json").unwrap();
    headers.insert("Content-Type",header_val);
    return headers;
}


async fn handle_success_request(res: reqwest::Response, ln_info:ChannelMessage, command:&str, send_tel:Sender<ChannelMessage>) -> Result<(), reqwest::Error>{
    let text = res.text().await?;
    let ln_response: LnGetInfo = serde_json::from_str(&text)
                                            .unwrap();
    let to_send = UserInfoLn {
        commit_hash: ln_response.commit_hash, 
        identity_pubkey: ln_response.identity_pubkey, 
        alias: ln_response.alias, 
        num_pending_channels: ln_response.num_pending_channels, 
        num_active_channels: ln_response.num_active_channels, 
        num_inactive_channels: ln_response.num_inactive_channels,
        num_peers: ln_response.num_peers
    };

    let tel_message = ChannelMessage {
        channel_type: ChannelType::TEL,
        command:command.to_string(),
        message:to_send.to_string(),
        node_url: "".to_string(),
        chat_id:ln_info.chat_id,
        macaroon:"".to_string()
    };

    info!("(handle_success_request) tel_message: {}", tel_message);
    if let Err(e) = send_tel.send(tel_message).await {
        error!("(handle_success_request) channel send error: {0}", e);
    }
    Ok(())
}


async fn handle_request_err(res: reqwest::Response, ln_info:ChannelMessage, command:&str, send_tel:Sender<ChannelMessage>) -> Result<(), reqwest::Error>{
    let text = res.text().await?;
    let tel_message = ChannelMessage {
        channel_type: ChannelType::TEL,
        command:command.to_string(),
        message: text,
        node_url: "".to_string(),
        chat_id:ln_info.chat_id,
        macaroon:"".to_string()
    };
    info!("handle_request_err");
    info!("(handle_request_err) tel_message: {}", tel_message);
    if let Err(e) = send_tel.send(tel_message).await {
        error!("(handle_request_err) channel send error {0}", e);
    }
    Ok(())
}

