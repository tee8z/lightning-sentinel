use log::{info, LevelFilter};
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
use tor_proxy::{tor, watch, clear_old_tor_log};
use pickle_jar::PickleJar;


#[tokio::main]
async fn main() -> Result<()> {
    clear_old_tor_log();
    setup_logger();

    tokio::spawn(async move {
        tor(config_wrapper::SETTINGS.socks_port_local);
    });

    watch()
        .unwrap();

    let pickle = PickleJar::init();
    let local_access = PickleJar::new(Arc::clone(&pickle.db));

    let channels = Messages::new();
    let (send_lnd, mut recieve_ln) = channels.lightning_messages;
    let (send_tel, mut recieve_tel) = channels.telegram_messages;

    let telegram_client = telegram_client::setup_client(&SETTINGS);
    let lnd_client = lightning_client::setup_client(&SETTINGS);

    
    let telegram_client_clone = telegram_client.clone();

    //NOTE: Listens for messages to send to telegram based on LN listening thread's responses
    tokio::spawn(async move {
        info!("(main) recieve_tel setup");
        while let Some(tel_info) =  recieve_tel.recv().await {
                info!("(main) recieve_tel: {}", tel_info);
                let send_message = objects::SendMessage{
                    chat_id: tel_info.chat_id,
                    text: tel_info.message
                };
                info!("(main) send_message: {}", send_message);
                telegram_client::send_message(telegram_client.clone(), &SETTINGS, send_message)
                                .await
                                .unwrap();
              }
    });

    //NOTE: Listens for requests to send to user's lightning nodes based on requests from telegram messages
    tokio::spawn(async move {
        while let Some(ln_info) = recieve_ln.recv().await {
                info!("(main) recieve_ln: {}", ln_info);
                let lnd_client_cl = lnd_client.clone();
                let picklejar_cp = PickleJar::new(Arc::clone(&pickle.db));
                let send_tel_cl = send_tel.clone();
                //NOTE: creates new thread to poll a user's ln node
                tokio::spawn(async move {
                    lightning_client::check_hidden_service(&lnd_client_cl,
                                                            ln_info, 
                                                            PickleJar::new(Arc::clone(&picklejar_cp.db)), 
                                                            send_tel_cl.clone())
                                                                    .await;
                });
        }
    });
    let poll_db = PickleJar::new(Arc::clone(&local_access.db));

    let initial_user = local_access.get_values()
                                .await;

    let send_lnd_cp = send_lnd.clone();

    //NOTE: Sets up initial threads of already registered users
    if initial_user.len() > 0 {
            for user in initial_user {
                let ln_info = ChannelMessage {
                    channel_type: ChannelType::LN,
                    chat_id:user.telegram_chat_id,
                    node_url:user.node_url,
                    command: "".to_string(),
                    message: "".to_string(),
                    macaroon: user.macaroon
                };

                send_lnd.clone()
                        .send(ln_info)
                        .await
                        .unwrap();
            }
    }

    //NOTE: Makes one task to poll the telegram bot for new messages
    telegram_client::poll_messages(telegram_client_clone, 
                                    &*SETTINGS, 
                                    send_lnd_cp.clone(), 
                                    PickleJar::new(Arc::clone(&poll_db.db)))
                    .await?;

    Ok(())
}

fn setup_logger() {
    let mut builder = env_logger::Builder::new();
    builder.filter(None, LevelFilter::Info);
    builder.target(Target::Stdout);
    builder.init();
}