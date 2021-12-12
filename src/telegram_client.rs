use crate::channel_handler::{LnInfo};
use crate::configuration;
use tokio::sync::mpsc::{Sender};
use tokio::time::{Instant, Duration, interval_at};
use reqwest::{
    StatusCode,
    header::{HeaderMap, HeaderValue}
};
use crate::client_wrapper::{ClientWrapper, build_url};
use anyhow::Result;


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
    let mut interval = interval_at(start, Duration::from_secs(30));
    loop {
        interval.tick().await;
        let sender = send_tel.clone();
        get_message(&client, settings, sender).await?;
    }
}

async fn get_message<'a>(client: &ClientWrapper, settings: &configuration::Settings, send_tel:Sender<LnInfo>) -> Result<(), reqwest::Error>{
    let base_url = build_full_base(settings);

    let res = client.client
        .get(build_url(base_url,"/getUpdates"))
        .headers(build_headers())
        .send()
        .await?;

    println!("Status: {}", res.status());

    let text = res.text().await?;

    println!("Response: {}", text);
    //TODO: pull from key/value file userId -> node_url &macaroon mapping made at "registration";

    let tel_info = LnInfo{
        node_url:"".to_string(),
        command:"".to_string(),
        is_active:true,
        message:text,
    };

    if let Err(e) = send_tel.send(tel_info).await {
        eprintln!("{0}", e);
    }

    Ok(())
}

pub fn build_full_base(settings: &configuration::Settings) -> String{
    return settings.telegram_base_url.to_string()+&settings.telegram_bot_id.to_owned();
}

/*
pub async fn send_message<'a>(settings: &configuration::Settings,client: ClientWrapper<'a>, recieve_ln: LnInfo) ->  Result<(), reqwest::Error>{
    let base_url = build_full_base(settings);

    let message = json::parse(
    format!("{""chat_id"":""{0}"",\n\r\"text\":\"{1}\",\n\r}",2222, recieve_ln.message);

    let res = client.client
        .post(client.build_url(base_url,"/getMe"))
        .body(message)
        .headers(*client.headers)
        .send()
        .await?;

    println!("Status: {}", res.status());

    let text = res.text().await?;

    println!("Response: {}", text);

    Ok(())

} */ 




