use once_cell::sync::Lazy;
use lightning_sentinel::telemetry::{get_subscriber};
use wiremock::MockServer;

static TRACING: Lazy<()> = Lazy::new(|| async {
    let default_filter_level = "info".to_string();
    if std::env::var("TEST_LOG").is_ok(){
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});


pub struct App {
    pub address: String,
    pub port: u16,
}

pub async fn spawn_app() -> App {
    Lazy::force(&TRACING);
    let lnd_server = MockServer::start().await;
    let configuration = {

    };
    
}