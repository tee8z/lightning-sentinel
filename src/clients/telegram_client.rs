use std::collections::HashMap;
use crate::channels::{ChannelMessage, ChannelType};
use crate::config_wrapper;
use tokio::sync::mpsc::{Sender};
use tokio::time::{Instant, Duration, interval_at};
use reqwest::{
    StatusCode,
    header::{HeaderMap, HeaderValue}
};
use super::client_wrapper::{ClientWrapper, build_url};
use anyhow::Result;
use crate::objects::{SendMessage, Update};
use crate::config_wrapper::Settings;



pub fn setup_client(settings: &Settings) -> ClientWrapper {
    let client = ClientWrapper::new(settings);
    return client;
}

fn build_headers() -> HeaderMap{
    let mut headers = HeaderMap::new();
    let header_val = HeaderValue::from_str(&"application/json").unwrap();
    headers.insert("Content-Type",header_val);
    return headers;
}

//TODO: implement long polling
pub async fn poll_messages<'a>(client: ClientWrapper, settings: &Settings, send_tel:Sender<ChannelMessage>) -> Result<(), reqwest::Error>{
    let start = Instant::now() + Duration::from_secs(20);
    let mut interval = interval_at(start, Duration::from_secs(60));
    loop {
        interval.tick().await;
        let sender = send_tel.clone();
        get_message(&client, settings, sender).await?;
    }
}

async fn get_message<'a>(client: &ClientWrapper, settings: &Settings, send_ln:Sender<ChannelMessage>) -> Result<(), reqwest::Error>{
    let base_url = build_full_base(settings);

    let res = client.client
        .get(build_url(base_url,"/getUpdates"))
        .headers(build_headers())
        .send()
        .await?;

    println!("Status: {}", res.status());

    let last_messages = res.json::<Vec<Update>>().await.unwrap();
    println!("Response: {:#?}", last_messages);
    //TODO: pull from key/value file userId -> node_url &macaroon mapping made at "registration";
    
   for update in last_messages {
        let ln_info = build_message(update);
        println!("{}", ln_info);
        if let Err(_) = send_ln.send(ln_info).await {
            println!("receiver dropped");
        }
        println!("After Send");

    };

    Ok(())
}

fn build_message(update: Update) -> ChannelMessage {
    let mut chat_id = -1;
    let mut text:String = "".to_string(); 
    match update.message {
        Some(mes) => {
            chat_id = mes.chat.id;
            match mes.text {
                Some(t) => {
                    text = t;
                }
                None => {}
            }
        }
        None => {}
    }

    //TODO: call in-memory collection for node_url
    //let row = 

    let ln_info = ChannelMessage{
        channel_type: ChannelType::LN,
        chat_id:chat_id,
        node_url:"".to_string(),
        command:"".to_string(),
        message:text,
        macaroon:"".to_string()
    };

    return ln_info;
}




pub fn build_full_base(settings: &Settings) -> String{
    return settings.telegram_base_url.to_string()+&settings.telegram_bot_id.to_owned();
}

pub async fn send_message(client: ClientWrapper, settings: &Settings, recieve_tn: ChannelMessage) ->  Result<(), reqwest::Error>{
    let base_url = build_full_base(settings);
    let message = SendMessage{
        chat_id: recieve_tn.chat_id,
        text: recieve_tn.message
    };

    let message_send = build_url(base_url,"/sendMessage");

    let res = client.client
        .post(message_send)
        .headers(build_headers())
        .json(&message)
        .send()
        .await?;

    println!("Status: {}", res.status());

    let text = res.text().await?;

    println!("Response: {}", text);

    Ok(())

}