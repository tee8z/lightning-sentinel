use log::{info,error};
use anyhow::Result;

mod tor_proxy;
mod clients;
mod objects;
mod pickledb_wrapper;
mod channels;
mod config_wrapper;

use channels::Messages;
use clients::telegram_client;
use clients::lightning_client;
use config_wrapper::SETTINGS;
use tor_proxy::tor_proxy::tor;
use pickledb_wrapper::Pickle;

//TODO: make interval of polling dependant on user's settings

#[tokio::main]
async fn main() -> Result<()> {
    
    tokio::spawn(async move {
        tor(config_wrapper::SETTINGS.socks_port_local);
    });
    
    let db = Pickle::new();
    
    let channels = Messages::new();
    let (send_lnd, mut recieve_ln) = channels.lightning_messages;
    let (send_tel, mut recieve_tel) = channels.telegram_messages;

    let telegram_client = telegram_client::setup_client(&SETTINGS);
    let lnd_client = lightning_client::setup_client(&SETTINGS);

    tokio::spawn(async move {
        match recieve_ln.recv().await {
            //SHOULD send response to telegram channel if:
            // 1) message was requested by the user
            // 2) there was an error (ie lightning node is down or some channels are inactive)
            //SHOULD not send response to telegram if:
            // - Regular pin, everything up/fine, not requested by user
            Some(ln_info) =>{ 
                println!("recieve_ln: {}", ln_info);
                lightning_client::check_hidden_service(&lnd_client,
                                                        ln_info, 
                                                        db, 
                                                        send_tel)
                                .await
                                .unwrap();
            },
            None => { error!("{}", "The message was never sent"); }
        };
    });
    let telegram_client_clone = telegram_client.clone();

    tokio::spawn(async move {
        println!("recieve_tel setup");
        match recieve_tel.recv().await {
            Some(tel_info) => {
                println!("recieve_tel: {}", tel_info);
                telegram_client::send_message(telegram_client, &SETTINGS, tel_info)
                                .await
                                .unwrap();
            }
            None => { error!("{}", "The message was never sent"); }
        };
    });
    //TODO: At startup, load registered users and kick off watching their hiddens services (in another task)

   /*tokio::spawn(async move {
        // Load user tor service watchers here
    })*/

    //NOTE: Should be one task polling the telegram bot for new messages
    telegram_client::poll_messages(telegram_client_clone, &*SETTINGS, send_lnd)
                    .await?;

                    
    Ok(())
}