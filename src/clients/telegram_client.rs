use crate::channels::{ChannelMessage, ChannelType, ThreadsMap};
use crate::objects::{SendMessage, Update};
use crate::config_wrapper::Settings;
use crate::pickle_jar::{PickleJar, Row};
use super::client_wrapper::{ClientWrapper, build_url};
use tokio::sync::mpsc::{Sender};
use tokio::time::{Instant, Duration, interval_at};
use reqwest::{
    header::{HeaderMap, HeaderValue}
};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use log::{info,error};
use std::{
    sync::{Arc, atomic::{AtomicU32, Ordering}},
    cmp::Reverse};
use lazy_static::lazy_static;
use regex::Regex;

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

pub async fn poll_messages(client: ClientWrapper, settings: &Settings, send_ln:Sender<ChannelMessage>, pickle:PickleJar, ln_threads:ThreadsMap) -> Result<(), reqwest::Error>{
    let start = Instant::now() + Duration::from_secs(20);
    let mut interval = interval_at(start, Duration::from_secs(20));
    loop {
        interval.tick().await;
        get_message(&client, settings, send_ln.clone(),
        PickleJar::new(Arc::clone(&pickle.db)), 
        ThreadsMap::new(Arc::clone(&ln_threads.ln_calling_threads))).await?;
    }
}
#[derive(Serialize, Deserialize, Debug)]
struct Response {
    pub ok: bool,
    pub result: Vec<Update>
}


static LAST_UPDATE: AtomicU32 = AtomicU32::new(u32::MIN);

fn set_update(update_id: u32) -> Result<u32, u32>{
    let cur = LAST_UPDATE.load(Ordering::SeqCst);
    LAST_UPDATE.compare_exchange(cur,update_id,Ordering::SeqCst, Ordering::Acquire)
}


async fn get_message(client: &ClientWrapper, settings: &Settings, send_ln:Sender<ChannelMessage>,pickle:PickleJar, ln_threads:ThreadsMap) -> Result<(), reqwest::Error>{
    let base_url = build_full_base(settings);

    let command_url;

    if LAST_UPDATE.load(Ordering::SeqCst) > 0 {
        command_url = format!("/getUpdates?offset={}&allowed_updates=[\"message\",\"callback_query\"]", LAST_UPDATE.load(Ordering::SeqCst)+1);
    }
    else{
        command_url = "/getUpdates?allowed_updates=[\"message\",\"callback_query\"]".to_string();
    }
    println!("command_url: {}",command_url);
    let res = client.client
        .get(build_url(base_url, &command_url))
        .headers(build_headers())
        .send()
        .await?;

    info!("Status: {}", res.status());

    let last_messages = res.json::<Response>().await.unwrap();
    info!("Response: {:#?}", last_messages);

    let mut update_ids = vec![];

    for update in last_messages.result {
        let update_id = update.update_id.clone();
        update_ids.push(update_id);

       handle_message(
                    client.clone(),
                    update.message
                            .clone()
                            .unwrap()
                            .chat.id, 
                      update.message
                            .unwrap()
                            .text
                            .unwrap(),
                    settings,
                    PickleJar::new(Arc::clone(&pickle.db)), 
                    ThreadsMap::new(Arc::clone(&ln_threads.ln_calling_threads)),
                    send_ln.clone())
                    .await;
    };

    update_ids.sort_by_key(|update_id| Reverse(*update_id));

    if update_ids.len() == 0 {
       return Ok(());
    }

    let last_message_time = update_ids[0];
    println!("last message time: {}", last_message_time);

    if last_message_time > LAST_UPDATE.load(Ordering::SeqCst){
        set_update(last_message_time).unwrap();
    }

    Ok(())
}

async fn handle_message(client: ClientWrapper, 
                        chat_id: i64, 
                        message: String, 
                        settings:&Settings, 
                        pickle:PickleJar,
                        ln_threads:ThreadsMap, 
                        send_ln:Sender<ChannelMessage>) {
    let address_mac = parse_address_token(&message);
    let parse = info_messages(message);
    let parse_cl = parse.clone();

    if address_mac.0.len() > 0 || parse.0 == "status" {
        let ln_info = build_message(chat_id.clone(), address_mac.0.clone(), address_mac.1.clone(), parse.0.clone());
        if address_mac.0.len() > 0 {
            let row = Row{
                telegram_chat_id:chat_id.clone(), 
                node_url: address_mac.0.clone(),
                macaroon: address_mac.1.clone(),
                is_watching: true,
            };
            pickle.set(&chat_id.to_string(), row)
                  .await;
        }


        if let Err(_) = send_ln.send(ln_info)
            .await {
            error!("receiver dropped");
        }
    }
    else if parse_cl.0 == "help" || parse_cl.0 == "start" || parse_cl.0 == "bad" || parse.0.clone() == "stop" {
        if parse.0.clone() == "stop"{
            ln_threads.cancel(chat_id);
            pickle.remove(&chat_id.to_string())
                  .await;
        }
        let message = SendMessage {
            chat_id: chat_id,
            text: parse_cl.1
        };
        send_message(client, settings, message)
        .await
        .unwrap();
        
        return;
    }
    return;

}

fn info_messages(action: String) -> (String,String) {
    match action.as_str() {
        "/start" => {
            return ("start".to_string(),r#"Signup by sending:
1) Tor address of your lighting node's REST api
2) Macaroon with the permissions to /getInfo endpoint
    
Reply to this message with a tuple, ex:
    (<lightning REST API tor address>,<macaroon>)
 
NOTE: Please look at this bot's README for details on obtaining these values, & other instructions on setup, if these values are new to you. The bot's README can be found here: https://github.com/tee8z/llightning-sentinel/blob/main/README.md"#.to_string())
        },
        "/help" => {
            return ("help".to_string(),r#"
To use this bot please use one of the following commands:
    
- /start - register node & start sentinel watching process
- /help - list of commands
- /status - status of registered node
- /stop - deregister node sentinel and delete data
            "#.to_string());
        }
        "/status" => {
            return ("status".to_string(),r#"Status of your node:
Active: {}
Channels: 
            {}
                       "#.to_string());
        },
        "/stop" => {
            return ("stop".to_string(),r#"Your data has been removed and your sentinel has stood down"#.to_string());
        },
        _ => {
            return ("bad".to_string(),r#"Please try one of the available options, your message was not understood:
- /start - register node
- /help - list of commands
- /status - status of registered node
- /stop - deregister node sentinel and delete data"#.to_string());

        }
    }
}

fn parse_address_token(message: &str) -> (String, String){
    lazy_static! {
        static ref USER_INFO: Regex = Regex::new(r"(https://(\S+)onion:\d{4}),(\S{258})").unwrap();
    }
  
    if USER_INFO.is_match(message){
        let found_data = USER_INFO.captures(message).unwrap();
        let lightning_add = found_data.get(1).map_or("", |m| m.as_str());
        info!("lightning_add: {}", lightning_add);
        let macaroon = found_data.get(3).map_or("", |m| m.as_str());
        info!("macaroon: {}", macaroon);
        return (lightning_add.to_string(), macaroon.to_string());
    }

    return ("".to_string(), "".to_string());
}   


fn build_message(chat_id:i64, node_url:String, macaroon:String, ln_command:String) -> ChannelMessage {

    let ln_info = ChannelMessage {
        channel_type: ChannelType::LN,
        chat_id:chat_id,
        node_url:node_url.clone(),
        macaroon:macaroon,
        command: if node_url.len() > 0 { "status".to_string() } else { ln_command },
        message:"".to_string(),
    };

    return ln_info;
}


pub fn build_full_base(settings: &Settings) -> String{
    return settings.telegram_base_url.to_string()+&settings.telegram_bot_id.to_owned();
}

pub async fn send_message(client: ClientWrapper, settings: &Settings, message:SendMessage) ->  Result<(), reqwest::Error>{
    let base_url = build_full_base(settings);

    let message_send = build_url(base_url,"/sendMessage");

    let res = client.client
        .post(message_send)
        .headers(build_headers())
        .json(&message)
        .send()
        .await?;

    info!("Status: {}", res.status());

    let text = res.text().await?;

    info!("Response: {}", text);

    Ok(())

}