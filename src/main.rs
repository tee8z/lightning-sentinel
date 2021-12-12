mod configuration;
mod tor_proxy;
mod lightning_client;
mod telegram_client;
mod channel_handler;
mod client_wrapper;

use log::{info,error};
use crate::configuration::SETTINGS;
use anyhow::Result;

//TODO: make interval of polling dependant on user's settings

#[tokio::main]
async fn main() -> Result<()> {
    
    tokio::spawn(async move {
        tor_proxy::tor(SETTINGS.socks_port_local);
    });

    let channels = channel_handler::Messages::new();
    let (send_lnd, mut recieve_ln) = channels.lightning_messages;
    let (send_tel, mut recieve_tel) = channels.telegram_messages;

    info!("Before call clients");

    let telegram_client = telegram_client::setup_client(&SETTINGS);
    let lnd_client = lightning_client::setup_client(&SETTINGS);
    //let send_ln_copy = send_ln.clone();
    let send_tel_copy = send_tel.clone();
    
    //NOTE: Should be one task polling the telegram bot for new messages
    telegram_client::poll_messages(telegram_client, &*SETTINGS, send_lnd).await?;

    //TODO: At startup, load registered users and kick off watching their hiddens services

    match recieve_ln.recv().await {
        //SHOULD send response to telegram channel if:
        // 1) message was requested by the user
        // 2) there was an error (ie lightning node is down or some channels are inactive)
        //SHOULD not send response to telegram if:
        // - Regular pin, everything up/fine, not requested by user
        Some(ln_info) =>{ 
            info!("{}", ln_info);
            lightning_client::check_hidden_service(&lnd_client, &SETTINGS.check_url, &SETTINGS.macaroon, send_tel_copy).await?;
        },
        None => { error!("{}", "The message was never sent"); }
    };

    match recieve_tel.recv().await {
        Some(tel_info) => {
            info!("{}", tel_info);
            //telegram_client::send_message();
            //lightning_client::check_hidden_service(&lnd_client, &SETTINGS.check_url, &SETTINGS.macaroon, send_tel_copy).await?;
        }
        None => { error!("{}", "The message was never sent"); }
    }

    Ok(())
}