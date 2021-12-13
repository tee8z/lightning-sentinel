use crate::channels::{ChannelMessage, ChannelType};
use crate::objects::{SendMessage, Update};
use crate::config_wrapper::Settings;
use super::client_wrapper::{ClientWrapper, build_url};
use tokio::sync::mpsc::{Sender};
use tokio::time::{Instant, Duration, interval_at};
use reqwest::{
    header::{HeaderMap, HeaderValue}
};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use log::{info,error};
use std::cmp::Reverse;
use std::sync::atomic::{AtomicU32, Ordering};


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

pub async fn poll_messages<'a>(client: ClientWrapper, settings: &Settings, send_tel:Sender<ChannelMessage>) -> Result<(), reqwest::Error>{
    let start = Instant::now() + Duration::from_secs(20);
    let mut interval = interval_at(start, Duration::from_secs(60));
    loop {
        interval.tick().await;
        let sender = send_tel.clone();
        get_message(&client, settings, sender).await?;
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


//TODO: pipe the messages depending on text recieved from user
async fn get_message<'a>(client: &ClientWrapper, settings: &Settings, send_ln:Sender<ChannelMessage>) -> Result<(), reqwest::Error>{
    let base_url = build_full_base(settings);

    let command_url;
    if LAST_UPDATE.load(Ordering::SeqCst) > 0 {
        command_url = format!("/getUpdates?offset={}&allowed_updates=[\"message\",\"callback_query\"]", LAST_UPDATE.load(Ordering::SeqCst));
    }
    else{
        command_url = "/getUpdates?allowed_updates=[\"message\",\"callback_query\"]".to_string();
    }

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

        let ln_info = build_message(update);
        info!("{}", ln_info);
        if let Err(_) = send_ln.send(ln_info).await {
            error!("receiver dropped");
        }
        info!("After Send");
    };

    update_ids.sort_by_key(|update_id| Reverse(*update_id));

    let last_message_time = update_ids[0];

    if last_message_time > LAST_UPDATE.load(Ordering::SeqCst){
        set_update(last_message_time).unwrap();
    }

    Ok(())
}

fn info_messages(action: String) -> String {
    let signup = String::from("signup");
    let status = String::from("status");
    let remove = String::from("remove");

    match action {
        signup => {
            return r#"Signup by sending:
                        1) tor address of your lighting node,
                        2) macaroon with the permissions to /getInfo endpoint
                    Reply to this message with a tuple (<lightning REST API tor address>,<macaroon>), ex:
                            (https://<wkdirllfgoofflfXXXXXXXXXXXXXXXXXXXXXXXXXXXXJJJJJJJJJJJJ.onion>:8080,XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY)
                    NOTE: Please look at this bot's README for details on obtaining these values,
                     & other instructions on setup, if these values are new to you. The bot's README
                     can be found here: https://github.com/tee8z/llightning-sentinel/blob/main/README.md"#.to_string()
        },
        status => {
            return r#"Status of your node:
                        Active: {}
                        Channels: 
                            {}
                       "#.to_string()
        },
        remove => {
            return r#"Your data has been removed and your sentinel has stood down"#.to_string()
        }
        _ => {
           return r#"Please try one of the available options, your message was not understood:
                    - signup
                    - status
                    - remove"#.to_string()
        }
    }
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


    //TODO, pick up node_url and macaroon from user answer and add to message
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

    info!("Status: {}", res.status());

    let text = res.text().await?;

    info!("Response: {}", text);

    Ok(())

}