use super::client_wrapper::{build_url, ClientWrapper};
use crate::channels::{ChannelMessage, ChannelType};
use crate::config_wrapper::Settings;
use crate::objects::{SendMessage, Update};
use crate::pickle_jar::{PickleJar, Row};
use anyhow::Result;
use lazy_static::lazy_static;
use log::{error, info};
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Reverse,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
};
use tokio::sync::mpsc::Sender;
use tokio::time::{interval, Duration};

pub fn setup_client(settings: &Settings) -> ClientWrapper {
    ClientWrapper::new(settings)
}

fn build_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    let header_val = HeaderValue::from_str("application/json").unwrap();
    headers.insert("Content-Type", header_val);
    headers
}
//TODO: create a message for when the tor process completes setup, then start polling instead of doing this wait time (prone to error)
pub async fn poll_messages(
    client: ClientWrapper,
    settings: &Settings,
    send_ln: Sender<ChannelMessage>,
    pickle: PickleJar,
) -> Result<(), reqwest::Error> {
    let mut interval = interval(Duration::from_secs(20));
    loop {
        interval.tick().await;
        match get_message(
            &client,
            settings,
            send_ln.clone(),
            PickleJar::new(Arc::clone(&pickle.db)),
        )
        .await {
            Ok(_) => { info!("(poll_messages) get_message successfull")}
            Err(e) => { info!("(poll_messages) get_message error {:#?}", e)}
        }
        
    }
}
#[derive(Serialize, Deserialize, Debug)]
struct Response {
    pub ok: bool,
    pub result: Vec<Update>,
}

static LAST_UPDATE: AtomicU32 = AtomicU32::new(u32::MIN);

fn set_update(update_id: u32) -> Result<u32, u32> {
    let cur = LAST_UPDATE.load(Ordering::SeqCst);
    LAST_UPDATE.compare_exchange(cur, update_id, Ordering::SeqCst, Ordering::Acquire)
}

async fn get_message(
    client: &ClientWrapper,
    settings: &Settings,
    send_ln: Sender<ChannelMessage>,
    pickle: PickleJar,
) -> Result<(), reqwest::Error> {
    let base_url = build_full_base(settings);

    let command_url;

    if LAST_UPDATE.load(Ordering::SeqCst) > 0 {
        command_url = format!(
            "/getUpdates?offset={}&allowed_updates=[\"message\",\"callback_query\"]",
            LAST_UPDATE.load(Ordering::SeqCst) + 1
        );
    } else {
        command_url = "/getUpdates?allowed_updates=[\"message\",\"callback_query\"]".to_string();
    }
    info!("(get_message) command_url: {}", command_url);
    let res;
    
    match client
        .client
        .get(build_url(base_url, &command_url))
        .headers(build_headers())
        .send()
        .await {
            Ok(response) => {
                res = response;
            }
            Err(e) => {
                info!("(get_message) error {:#?}", e);
                return Ok(())
            }
        }

    info!("(get_message) status: {}", res.status());
    let last_messages;

    match res.json::<Response>().await {
        Ok(last_message_reponse) => { last_messages = last_message_reponse}
        Err(e) => {
            info!("(get_message) error calling telegram bot: {:#?}", e);
            return Ok(())
        }
    }

    info!("(get_message) last_messages: {:#?}", last_messages);

    let mut update_ids = vec![];

    for update in last_messages.result {
        let update_id = update.update_id;
        update_ids.push(update_id);

        handle_message(
            client.clone(),
            update.message.clone().unwrap().chat.id,
            update.message.unwrap().text.unwrap(),
            settings,
            PickleJar::new(Arc::clone(&pickle.db)),
            send_ln.clone(),
        )
        .await;
    }

    update_ids.sort_by_key(|update_id| Reverse(*update_id));

    if update_ids.is_empty() {
        return Ok(());
    }

    let last_message_time = update_ids[0];
    println!("(get_message) last_message_time: {}", last_message_time);

    if last_message_time > LAST_UPDATE.load(Ordering::SeqCst) {
        match set_update(last_message_time) {
            Ok(_) => { info!("(get_message) set_update successful")}
            Err(err) => {
                info!("(get_message) set_update error {:#?}", err);
                return Ok(())
            }
        }
    }

    Ok(())
}

async fn handle_message(
    client: ClientWrapper,
    chat_id: i64,
    message: String,
    settings: &Settings,
    pickle: PickleJar,
    send_ln: Sender<ChannelMessage>,
) {
    let address_mac = parse_address_token(&message);
    let parse = info_messages(message);
    let parse_cl = parse.clone();

    if !address_mac.0.is_empty() || parse.0 == "status" {
        let command = if !address_mac.0.is_empty() {
            "start".to_string()
        } else {
            "status".to_string()
        };
        let ln_info = build_message(
            chat_id,
            address_mac.0.clone(),
            address_mac.1.clone(),
            command,
        );
        if !address_mac.0.is_empty(){
            let row = Row {
                telegram_chat_id: chat_id,
                node_url: address_mac.0.clone(),
                macaroon: address_mac.1.clone(),
                is_watching: true,
            };

            pickle.set(&chat_id.to_string(), row).await;
        }

        if send_ln.send(ln_info).await.is_err() {
            error!("(handle_message): receiver dropped");
        }
    } else if parse_cl.0 == "help"
        || parse_cl.0 == "start"
        || parse_cl.0 == "bad"
        || parse.0.clone() == "stop"
    {
        if parse.0.clone() == "stop" {
            let pickle_cp = PickleJar::new(Arc::clone(&pickle.db));
            let mut saved = pickle.get(&chat_id).await;
            saved.is_watching = false;
            pickle_cp.set(&chat_id.to_string(), saved).await;
        }
        let message = SendMessage {
            chat_id,
            text: parse_cl.1,
        };
        match send_message(client, settings, message).await {
            Ok(_) => {  }
            Err(e) => { info!("(handle_message) Error in sending message {:#?}", e); }
        }
    }
}

fn info_messages(action: String) -> (String, String) {
    match action.as_str() {
        "/start" => {
            ("start".to_string(),r#"Signup by sending:
1) Address of your lighting node's REST api (can be tor or clearnet)
2) Macaroon with the permissions to /getInfo endpoint
    
Reply to this message with a tuple, ex:
    https://<lightning REST API address>:<port>, <macaroon>
 
NOTE: Please look at this bot's README for details on obtaining these values, & other instructions on setup, if these values are new to you. The bot's README can be found here: https://github.com/tee8z/llightning-sentinel/blob/main/README.md"#.to_string())
        },
        "/help" => {
            ("help".to_string(),r#"
To use this bot please use one of the following commands:
    
- /start - register node & start sentinel watching process
- /help - list of commands
- /status - status of registered node
- /stop - deregister node sentinel and delete data
            "#.to_string())
        }
        "/status" => {
            ("status".to_string(),r#"Status of your node:
Active: {}
Channels: 
            {}
                       "#.to_string())
        },
        "/stop" => {
            ("stop".to_string(),r#"Your data has been removed and your sentinel has stood down"#.to_string())
        },
        _ => {
            ("bad".to_string(),r#"Please try one of the available options, your message was not understood:
- /start - register node
- /help - list of commands
- /status - status of registered node
- /stop - deregister node sentinel and delete data"#.to_string())

        }
    }
}
//TODO: add a unit test around this function to test patterns of user input
fn parse_address_token(message: &str) -> (String, String) {
    lazy_static! {
        static ref USER_INFO: Regex = Regex::new(r"https://(\S+):(\d+), (\S+)").unwrap();
    }

    if USER_INFO.is_match(message) {
        let found_data = USER_INFO.captures(message).unwrap();
        let lightning_add = found_data.get(1).map_or("", |m| m.as_str());
        info!("(parse_address_token) lightning_add: {}", lightning_add);
        let port = found_data.get(2).map_or("", |m| m.as_str());
        info!("(parse_address_token) port: {}", port);
        let macaroon = found_data.get(3).map_or("", |m| m.as_str());
        info!("(parse_address_token) macaroon: {}", macaroon);
        return (lightning_add.to_string()+":"+port, macaroon.to_string());
    }

    ("".to_string(), "".to_string())
}

fn build_message(
    chat_id: i64,
    node_url: String,
    macaroon: String,
    ln_command: String,
) -> ChannelMessage {
    ChannelMessage {
        channel_type: ChannelType::Ln,
        chat_id,
        node_url,
        macaroon,
        command: ln_command,
        message: "".to_string(),
    }
}
pub async fn send_message(
    client: ClientWrapper,
    settings: &Settings,
    message: SendMessage,
) -> Result<(), reqwest::Error> {
    let base_url = build_full_base(settings);

    let message_send = build_url(base_url, "/sendMessage");

    match client
        .client
        .post(message_send)
        .headers(build_headers())
        .json(&message)
        .send()
        .await {
            Ok(res) => { 
                info!("(send_message) status: {}", res.status());
                let text = res.text().await?;
                info!("(send_message) response: {}", text);
            }
            Err(e) => {
                error!("(send_message) error: {}", e.to_string());
            }
        }

    Ok(())
}

pub fn build_full_base(settings: &Settings) -> String {
    settings.telegram_base_url.to_string() + &settings.telegram_bot_id.to_owned()
}
