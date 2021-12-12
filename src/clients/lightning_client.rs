use tokio::time::{Instant, Duration, interval_at};
use crate::channels::{ChannelMessage, ChannelType};
use crate::config_wrapper::Settings;
use crate::pickledb_wrapper::Pickle;
use super::client_wrapper::{ClientWrapper, build_url};
use tokio::sync::mpsc::{Sender};
use reqwest::{
    StatusCode,
    header::{HeaderMap, HeaderValue}
};
use anyhow::Result;


pub fn setup_client(settings: &Settings) -> ClientWrapper {
    let client = ClientWrapper::new(settings);
    return client;
}

// Polling lightning node done here
pub async fn check_hidden_service(client: &ClientWrapper, ln_info: ChannelMessage, pickle: Pickle, send_tel:Sender<ChannelMessage>) -> Result<()>{
    let command = "/v1/getinfo";

    let row = pickle.get(&ln_info.chat_id)
                    .await;

    let mut node_url = "";
    let mut macaroon = "";
    
    let start = Instant::now() + Duration::from_secs(20);
    let mut interval = interval_at(start, Duration::from_secs(30));
    loop {
        interval.tick().await;
        let ln_info_clone = ln_info.clone();
        get_command_node(&client, ln_info_clone, ln_info.node_url.to_string(), ln_info.macaroon.to_string(), send_tel.clone(), command.to_string()).await?;
    }

}

//TODO: Clean response from node to be clear/simple to end user
pub async fn get_command_node(client: &ClientWrapper, ln_info: ChannelMessage, check_url:String, macaroon:String, send_tel:Sender<ChannelMessage>, command:String)-> Result<(), reqwest::Error> {
    
    let url = build_url(check_url, &command);
    println!("{0}", url);
    let res = client
    .client
    .get(url)
    .headers(build_headers(&macaroon))
    .send()
    .await?;

        println!("Status: {}", res.status());

        match res.status() {
            StatusCode::OK => {
                handle_success_request(res,ln_info, &command, send_tel)
                            .await
                            .unwrap();
            }
            StatusCode::CONTINUE => {
                handle_request_err(res,ln_info, &command, send_tel)
                            .await
                            .unwrap();
            }
        status => println!("status: {}", status),
        }
     Ok(())
}

fn build_headers(macaroon: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let header_val = HeaderValue::from_str(macaroon).unwrap();
    headers.insert("Grpc-Metadata-macaroon",header_val);
    return headers;
}

async fn handle_success_request(res: reqwest::Response, ln_info:ChannelMessage, command:&str, send_tel:Sender<ChannelMessage>) -> Result<(), reqwest::Error>{
    let text = res.text().await?;

    println!("Response: {}", text);
    let tel_message = ChannelMessage {
        channel_type: ChannelType::TEL,
        command:command.to_string(),
        message:text,
        node_url: "".to_string(),
        chat_id:ln_info.chat_id,
        macaroon:"".to_string()
    };
    println!("handle_success_request");
    println!("{}", tel_message);
    if let Err(e) = send_tel.send(tel_message).await {
        eprintln!("handle_success_request channel send error: {0}", e);
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
    println!("handle_request_err");
    println!("{}", tel_message);
    if let Err(e) = send_tel.send(tel_message).await {
        eprintln!("handle_request_err channel send error {0}", e);
    }
    Ok(())
}
