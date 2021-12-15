use crate::config_wrapper::Settings;
use log::{info};

#[derive(Debug, Clone)]
pub struct ClientWrapper{
    pub client:reqwest::Client,
}

//TODO: Implement Exponential_backoff logic and replace http request with that method

impl ClientWrapper{
    pub fn new(settings: &Settings) -> Self { 

        let proxy = reqwest::Proxy::all(build_poxy_location(settings))
        .expect("tor proxy should be there");

        return ClientWrapper
        {
            client: reqwest::Client::builder()
                .danger_accept_invalid_certs(true)
                .proxy(proxy)
                .build()
                .expect("should be able to build reqwest client")
        }
    }
    
}
pub fn build_poxy_location(settings: &Settings) -> String {
    let proxy_url = settings.sock_url_local.to_string();
    let proxy_port_str = settings.socks_port_local.to_string();
    let full_proxy = proxy_url.to_string() +":"+ &proxy_port_str;
    info!("build_poxy_location: {}", full_proxy);
    return full_proxy;
}   

pub fn build_url(base_url: String, command: &str) -> String{
    let full_url = base_url + command;
    info!("build_url: {}", full_url);
    return full_url;
}
