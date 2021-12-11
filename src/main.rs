use libtor::{HiddenServiceVersion, Tor, TorAddress, TorFlag};
use anyhow::Result;
use config;
use log::{debug, info, trace};
use reqwest::header::{HeaderMap, HeaderValue};
use std::time::Duration;

//TODO: on alert of failed request, send notification via telegram

struct LndTorSettings {
    pub sock_url_local: String,
    pub socks_port_local: u16,
    pub check_url: String,
    pub macaroon: String
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut settings = config::Config::default();
    settings
		.merge(config::File::with_name("Settings")).unwrap()
		.merge(config::Environment::with_prefix("APP")).unwrap();
    
    let settings = LndTorSettings {
        sock_url_local: settings.get_str("SOCKS_URL_LOCAL").unwrap(),
        socks_port_local: settings.get("SOCKS_PORT_LOCAL").unwrap(),
        check_url: settings.get_str("CHECK_URL").unwrap(),
        macaroon: settings.get("MACAROON").unwrap()
    };


    tokio::spawn(async move {
        tor(settings.socks_port_local);
    });

    let sleep_for = Duration::from_secs(10);
    loop {
        tokio::time::sleep(sleep_for).await;
        check_hidden_service(settings).await?;
    }

}



//blocking
fn tor(proxy_port: u16) {
    match Tor::new()
        .flag(TorFlag::DataDirectory("/tmp/tor-rust".into()))
        .flag(TorFlag::SocksPort(proxy_port))
        .flag(TorFlag::HiddenServiceDir("/tmp/tor-rust/hs-dir".into()))
        .flag(TorFlag::HiddenServiceVersion(HiddenServiceVersion::V3))
        .flag(TorFlag::HiddenServicePort(
            TorAddress::Port(8000),
            None.into(),
        ))
        .start()
    {
        Ok(r) =>{ println!("tor exit result: {}", r);},
        Err(e) =>{ eprintln!("tor error: {}", e); },
    };
}

async fn check_hidden_service(settings: LndTorSettings) -> Result<(), reqwest::Error> {

    let proxy = reqwest::Proxy::all(build_poxy_location(&settings.sock_url_local, settings.socks_port_local))
                             .expect("tor proxy should be there");


    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .proxy(proxy)
        .build()
        .expect("should be able to build reqwest client");

    let headers = build_headers(settings.macaroon);
    let res = client.get(build_url(&settings.check_url,"/v1/getinfo"))
                    .headers(headers.1)
                    .send()
                    .await?;

    println!("Status: {}", res.status());

    let text = res.text().await?;

    println!("Response: {}", text);

    Ok(())
}

fn build_poxy_location(proxy_url: &str, proxy_port: u16) -> String {
    let proxy_port_str = proxy_port.to_string();
    let full_proxy = proxy_url.to_string() +":"+ &proxy_port_str;
    println!("{}", full_proxy);
    return full_proxy;
}   

///v1/getinfo
fn build_url(url: &str, command: &str) -> String{
    let full_url = url.to_string() + command;
    println!("{}", full_url);
    return full_url;
}


fn build_headers<'a>(macaroon: &'a str) -> (&'a str, HeaderMap) {
    let mut headers = HeaderMap::new();
    let headerVal = HeaderValue::from_static(macaroon);
    headers.insert("Grpc-Metadata-macaroon",headerVal);
    return (macaroon, headers);
}