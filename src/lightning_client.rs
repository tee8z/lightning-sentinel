use tokio::time::{Instant, Duration, interval_at};
use crate::channel_handler::TelInfo;
use crate::configuration;
use crate::client_wrapper::{ClientWrapper, build_url};
use tokio::sync::mpsc::{Sender};
use reqwest::{
    StatusCode,
    header::{HeaderMap, HeaderValue}
};
use anyhow::Result;


pub fn setup_client(settings: &configuration::Settings) -> ClientWrapper {
    let client = ClientWrapper::new(settings);
    return client;
}

// Polling lightning node done here
pub async fn check_hidden_service(client: &ClientWrapper, check_url:&str, macaroon:&str, send_tel:Sender<TelInfo>) -> Result<()>{
    let command = "/v1/getinfo";
    let start = Instant::now() + Duration::from_secs(20);
    let mut interval = interval_at(start, Duration::from_secs(30));
    loop {
        interval.tick().await;
        get_command_node(&client, check_url.to_string(),macaroon.to_string(), send_tel.clone(), command.to_string()).await?;
    }
}


pub async fn get_command_node(client: &ClientWrapper, check_url:String, macaroon:String, send_tel:Sender<TelInfo>, command:String)-> Result<(), reqwest::Error> {
    
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
                handle_success_request(res, &command, send_tel)
                            .await
                            .unwrap();
            }
            StatusCode::CONTINUE => {
                handle_request_err(res, &command, send_tel)
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

async fn handle_success_request(res: reqwest::Response,command:&str, send_tel:Sender<TelInfo>) -> Result<(), reqwest::Error>{
    let text = res.text().await?;

    println!("Response: {}", text);
    let tel_message = TelInfo {
        user_id:"".to_string(),
        command:command.to_string(),
        is_active:true,
        message:text,
    };
    println!("handle_success_request");
    println!("{}", tel_message);
    if let Err(e) = send_tel.send(tel_message).await {
        eprintln!("handle_success_request channel send error: {0}", e);
    }
    Ok(())
}


async fn handle_request_err(res: reqwest::Response,command:&str, send_tel:Sender<TelInfo>) -> Result<(), reqwest::Error>{
    let text = res.text().await?;
    let tel_message = TelInfo {
        user_id:"".to_string(),
        command:command.to_string(),
        is_active:true,
        message:text,
    };
    println!("handle_request_err");
    println!("{}", tel_message);
    if let Err(e) = send_tel.send(tel_message).await {
        eprintln!("handle_request_err channel send error {0}", e);
    }
    Ok(())
}
