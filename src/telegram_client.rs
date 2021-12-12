use std::collections::HashMap;
use crate::channel_handler::{LnInfo, TelInfo};
use crate::configuration;
use tokio::sync::mpsc::{Sender};
use tokio::time::{Instant, Duration, interval_at};
use reqwest::{
    StatusCode,
    header::{HeaderMap, HeaderValue}
};
use crate::client_wrapper::{ClientWrapper, build_url};
use anyhow::Result;
use crate::telegram_objects::*;



pub fn setup_client(settings: &configuration::Settings) -> ClientWrapper {
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
pub async fn poll_messages<'a>(client: ClientWrapper, settings: &configuration::Settings, send_tel:Sender<LnInfo>) -> Result<(), reqwest::Error>{
    let start = Instant::now() + Duration::from_secs(20);
    let mut interval = interval_at(start, Duration::from_secs(60));
    loop {
        interval.tick().await;
        let sender = send_tel.clone();
        get_message(&client, settings, sender).await?;
    }
}

async fn get_message<'a>(client: &ClientWrapper, settings: &configuration::Settings, send_ln:Sender<LnInfo>) -> Result<(), reqwest::Error>{
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
        let mut chat_id = -1;
        let mut text:String = "".to_string(); 
        let mut user_id:String = "".to_string();
        match update.message {
            Some(mes) => {
                chat_id = mes.chat.id;
                match mes.text {
                    Some(t) => {
                        text = t;
                    }
                    None => {}
                }
                match mes.username {
                    Some(un) => {
                        user_id = un;
                    }
                    None => {}
                }
            }
            None => {}
        } 
        //TODO: 
        // determine what command to send to ln based on user messages
        // find users already registered node in key/value file

        let tel_info = LnInfo{
            node_url:"".to_string(),
            command:"".to_string(),
            is_active:true,
            message:text,
            chat_id:chat_id,
            user_id: user_id
        };
        println!("{}", tel_info);
        if let Err(_) = send_ln.send(tel_info).await {
            println!("receiver dropped");
        }
        println!("After Send");

    };

    Ok(())
}

pub fn build_full_base(settings: &configuration::Settings) -> String{
    return settings.telegram_base_url.to_string()+&settings.telegram_bot_id.to_owned();
}

pub async fn send_message(client: ClientWrapper, settings: &configuration::Settings, recieve_tn: TelInfo) ->  Result<(), reqwest::Error>{
    let base_url = build_full_base(settings);
    let message = Message{
        chat_id: recieve_tn.chat_id,
        text: &recieve_tn.message
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