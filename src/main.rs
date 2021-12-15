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

use channels::{Messages, ChannelMessage, ChannelType, ThreadsMap};
use clients::telegram_client;
use clients::lightning_client;
use config_wrapper::SETTINGS;
use tor_proxy::tor_proxy::tor;
use pickle_jar::PickleJar;
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> Result<()> {
    
    setup_logger();

    tokio::spawn(async move {
        tor(config_wrapper::SETTINGS.socks_port_local);
    });
    let ln_threads = ThreadsMap::init();
    let ln_thread_cp = ThreadsMap::new(Arc::clone(&ln_threads.ln_calling_threads));
    let ln_thread_cp2 = ThreadsMap::new(Arc::clone(&ln_threads.ln_calling_threads));

    let pickle = PickleJar::init();
    let local_access = PickleJar::new(Arc::clone(&pickle.db));

    let channels = Messages::new();
    let (send_lnd, mut recieve_ln) = channels.lightning_messages;
    let (send_tel, mut recieve_tel) = channels.telegram_messages;
    let send_tel_cl = send_tel.clone();

    let telegram_client = telegram_client::setup_client(&SETTINGS);
    let lnd_client = lightning_client::setup_client(&SETTINGS);

    
    let telegram_client_clone = telegram_client.clone();

    //NOTE: Listens for messages to send to telegram based on LN listening thread responses
    tokio::spawn(async move {
        info!("recieve_tel setup");
        while let Some(tel_info) =  recieve_tel.recv().await {
                info!("recieve_tel: {}", tel_info);
                let send_message = objects::SendMessage{
                    chat_id: tel_info.chat_id,
                    text: tel_info.message
                };
                info!("send_message: {}", send_message);
                telegram_client::send_message(telegram_client.clone(), &SETTINGS, send_message)
                                .await
                                .unwrap();
              }
    });

    //NOTE: Listens for requests to send to uer's lightning notes based on requests from telegram messages
    tokio::spawn(async move {
        while let Some(ln_info) = recieve_ln.recv().await {
                info!("recieve_ln: {}", ln_info);
                let token = CancellationToken::new();
                let ln_thread_lp = ThreadsMap::new(Arc::clone(&ln_threads.ln_calling_threads));
                ln_thread_lp.insert(ln_info.chat_id, token);

                lightning_client::check_hidden_service(&lnd_client,
                                                        ln_info, 
                                                        PickleJar::new(Arc::clone(&pickle.db)), 
                                                        send_tel.clone())
                                                                .await;
        }
    });
    let inital_db = PickleJar::new(Arc::clone(&local_access.db));
    let poll_db = PickleJar::new(Arc::clone(&local_access.db));

    let initial_user = local_access.get_values().await;

    let lnd_client_initial = lightning_client::setup_client(&SETTINGS);
    
    //NOTE: Sets up initial threads of already registered users
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
                let ln_thread_lp = ThreadsMap::new(Arc::clone(&ln_thread_cp.ln_calling_threads));
                let token = CancellationToken::new();
                ln_thread_lp.insert(ln_info.chat_id, token);

                lightning_client::check_hidden_service(&lnd_client_initial.clone(),
                    ln_info, 
                    PickleJar::new(Arc::clone(&inital_db.db)), 
                    send_tel_cl.clone()).await
            }
        });
    }

    //NOTE: Should be one task polling the telegram bot for new messages
    telegram_client::poll_messages(telegram_client_clone, 
                                    &*SETTINGS, 
                                    send_lnd.clone(), 
                                    PickleJar::new(Arc::clone(&poll_db.db)),
                                    ThreadsMap::new(Arc::clone(&ln_thread_cp2.ln_calling_threads)))
                    .await?;

    Ok(())
}

fn setup_logger() {
    let mut builder = env_logger::Builder::new();
    builder.filter(None, LevelFilter::Info);
    builder.target(Target::Stdout);
    builder.init();
}