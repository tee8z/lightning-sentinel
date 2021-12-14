use tokio::time::{Instant, Duration, interval_at};
use crate::channels::{ChannelMessage, ChannelType};
use crate::config_wrapper::Settings;
use crate::pickle_jar::{PickleJar};
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
use std::fmt;

#[derive(Serialize, Deserialize)]
struct UserInfoLn {
    pub version: String, 
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
            'version': '{}',
            'commit_hash': '{}',
            'identity_pubkey': '{}',
            'alias': '{}',
            'num_pending_channels': '{}',
            'num_active_channels': '{}'
            'num_inactive_channels': '{}'
            'num_peers': '{}'
        )"#, self.version, self.commit_hash, self.identity_pubkey, self.alias, self.num_pending_channels, self.num_active_channels, self.num_inactive_channels, self.num_peers);
    }
}

#[derive(Serialize, Deserialize)]
struct LnResponse { 
    pub version: String, 
    pub commit_hash: String, 
    pub identity_pubkey: String, 
    pub alias: String, 
    color: String, 
    pub num_pending_channels: i64, 
    pub num_active_channels: i64, 
    pub num_inactive_channels: i64, 
    pub num_peers: i64, 
    block_height: i64, 
    block_hash: String, 
    best_header_timestamp: String, 
    synced_to_chain: bool, 
    synced_to_graph: bool, 
    testnet: bool, 
    chains: Vec<LnrpcChain>, 
    uris: Vec<String>, 
    features: Vec<FeaturesEntry>
}

#[derive(Serialize, Deserialize)]
struct LnrpcChain {
    chain: String,
    network: String
}

#[derive(Serialize, Deserialize)]
struct FeaturesEntry {
    key: u32,
    value: Feature,
}

#[derive(Serialize, Deserialize)]
struct Feature {
    name: String,
    is_required: bool,
    is_known: bool
}

pub fn setup_client(settings: &Settings) -> ClientWrapper {
    let client = ClientWrapper::new(settings);
    return client;
}

// Polling lightning node done here
pub async fn check_hidden_service(client: &ClientWrapper, ln_info: ChannelMessage, pickle: PickleJar, send_tel:Sender<ChannelMessage>) {
    let command = "/v1/getinfo";
    
    let resolved_data = handle_check_service(ln_info.clone(), pickle.clone()).await;
    let start = Instant::now() + Duration::from_secs(20);
    let mut interval = interval_at(start, Duration::from_secs(30));
   
    loop {
        interval.tick().await;
        let url = &resolved_data.0;
        let macaroon = &resolved_data.1;
        match get_command_node(&client, ln_info.clone(), url.to_string(), macaroon.to_string(), send_tel.clone(), command.to_string()).await
        {
            _ => { return; }
        }

    }

}

pub async fn handle_check_service(ln_info: ChannelMessage, pickle: PickleJar) -> (String, String) {

    let row = pickle.get(&ln_info.chat_id)
                        .await;

    if row.is_watching {
        return ("".to_string(), "".to_string());
    }
    let mut node_url = row.node_url;

    if node_url.len() == 0 {
        node_url = ln_info.node_url;
    }

    let mut macaroon = row.macaroon;
    if macaroon.len() == 1 {
        macaroon = ln_info.macaroon;
    }
    
    return (node_url, macaroon);
}


//TODO: Clean response from node to be clear/simple to end user
pub async fn get_command_node<'a>(client: &ClientWrapper, ln_info: ChannelMessage, check_url:String, macaroon:String, send_tel:Sender<ChannelMessage>, command:String)-> Result<(), reqwest::Error> {
    
    let url = build_url(check_url, &command);
    info!("{0}", url);
    let res = client
                .client
                .get(url)
                .headers(build_headers(&macaroon))
                .send()
                .await?;

    info!("Status: {}", res.status());

        match res.status() {
            StatusCode::OK => {
                handle_success_request(res,ln_info, &command, send_tel.clone())
                            .await
                            .unwrap();
            }
            StatusCode::CONTINUE => {
                handle_request_err(res,ln_info, &command, send_tel.clone())
                            .await
                            .unwrap();
            }
        status => info!("status: {}", status),
        }
     Ok(())
}

fn build_headers(macaroon: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let header_val = HeaderValue::from_str(macaroon).unwrap();
    headers.insert("Grpc-Metadata-macaroon",header_val);
    return headers;
}

//TODO: SHOULD send response to telegram channel if:
//  1) message was requested by the user
//  2) there was an error (ie lightning node is down or some channels are inactive)
//SHOULD not send response to telegram if:
// - Regular pin, everything up/fine, not requested by user
async fn handle_success_request(res: reqwest::Response, ln_info:ChannelMessage, command:&str, send_tel:Sender<ChannelMessage>) -> Result<(), reqwest::Error>{
    let text = res.text().await?;
    let ln_response: LnResponse = serde_json::from_str(&text)
                                            .unwrap();
    let to_send = UserInfoLn {
        version: ln_response.version, 
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
    info!("handle_success_request");
    info!("{}", tel_message);
    if let Err(e) = send_tel.send(tel_message).await {
        error!("handle_success_request channel send error: {0}", e);
    }
    Ok(())
}


async fn handle_request_err(res: reqwest::Response, ln_info:ChannelMessage, command:&str, send_tel:Sender<ChannelMessage>) -> Result<(), reqwest::Error>{
    let text = res.text().await?;
    let tel_message = ChannelMessage {
        channel_type: ChannelType::TEL,
        command:command.to_string(),
        message:text,
        node_url: "".to_string(),
        chat_id:ln_info.chat_id,
        macaroon:"".to_string()
    };
    info!("handle_request_err");
    info!("{}", tel_message);
    if let Err(e) = send_tel.send(tel_message).await {
        error!("handle_request_err channel send error {0}", e);
    }
    Ok(())
}

