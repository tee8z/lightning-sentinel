extern crate log;

use log::{info,error, LevelFilter};
use env_logger::{Target};
use anyhow::Result;
use std::sync::{Arc};

mod tor_proxy;
mod clients;
mod objects;
mod pickle_jar;
mod channels;
mod config_wrapper;

use channels::{Messages, ChannelMessage, ChannelType};
use clients::telegram_client;
use clients::lightning_client;
use config_wrapper::SETTINGS;
use tor_proxy::tor_proxy::tor;
use pickle_jar::PickleJar;


#[tokio::main]
async fn main() -> Result<()> {
    
    setup_logger();

    tokio::spawn(async move {
        tor(config_wrapper::SETTINGS.socks_port_local);
    });

    let pickle = PickleJar::init();
    let local_access = PickleJar::new(Arc::clone(&pickle.db));

    let channels = Messages::new();
    let (send_lnd, mut recieve_ln) = channels.lightning_messages;
    let (send_tel, mut recieve_tel) = channels.telegram_messages;

    let send_tel_cp = send_tel.clone();

    let telegram_client = telegram_client::setup_client(&SETTINGS);
    let lnd_client = lightning_client::setup_client(&SETTINGS);

    tokio::spawn(async move {
        match recieve_ln.recv().await {
            Some(ln_info) =>{ 
                info!("recieve_ln: {}", ln_info);
                lightning_client::check_hidden_service(&lnd_client,
                                                        ln_info, 
                                                        PickleJar::new(Arc::clone(&pickle.db)), 
                                                        send_tel.clone())
                                .await;
            },
            None => { error!("{}", "The message was never sent"); }
        };
    });
    let telegram_client_clone = telegram_client.clone();

    tokio::spawn(async move {
        info!("recieve_tel setup");
        match recieve_tel.recv().await {
            Some(tel_info) => {
                info!("recieve_tel: {}", tel_info);
                telegram_client::send_message(telegram_client, &SETTINGS, tel_info)
                                .await
                                .unwrap();
            }
            None => { error!("{}", "The message was never sent"); }
        };
    });

    let inital_db = PickleJar::new(Arc::clone(&local_access.db));

    let initial_user = local_access.get_values().await;

    let lnd_client_initial = lightning_client::setup_client(&SETTINGS);
    
    if initial_user.len() > 0 {
        tokio::spawn(async move {
            for user in initial_user {
                let ln_info = ChannelMessage {
                    channel_type: ChannelType::LN,
                    chat_id:user.telegram_chat_id,
                    node_url:user.node_url,
                    command: "".to_string(),
                    message: "".to_string(),
                    macaroon: user.macaroon
                };
                lightning_client::check_hidden_service(&lnd_client_initial.clone(),
                    ln_info, 
                    PickleJar::new(Arc::clone(&inital_db.db)), 
                    send_tel_cp.clone()).await
            }
        });
    }

    //NOTE: Should be one task polling the telegram bot for new messages
    telegram_client::poll_messages(telegram_client_clone, &*SETTINGS, send_lnd)
                    .await?;

    Ok(())
}

fn setup_logger() {
    let mut builder = env_logger::Builder::new();
    builder.filter(None, LevelFilter::Info);
    builder.target(Target::Stdout);
    builder.init();
}