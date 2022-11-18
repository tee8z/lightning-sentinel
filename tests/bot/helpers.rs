use once_cell::sync::Lazy;
use lightning_sentinel::telemetry{get_subscriber}


static TRACING: Lazy<()> = Lazy::new(|| async {
    let default_filter_level = "info".to_string();
    if std::env::var("TEST_LOG").is_ok(){
        let subscriber = 
    }
})