use config::{Config, Environment, File};
use lazy_static::lazy_static;
lazy_static! {
    pub static ref SETTINGS: Settings = Settings::new();
}

pub struct Settings {
    pub sock_url_local: String,
    pub socks_port_local: u16,
    pub telegram_bot_id: String,
    pub telegram_base_url: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            sock_url_local: "socks5h://127.0.0.1".to_string(),
            socks_port_local: 9056,
            telegram_bot_id: "".to_string(),
            telegram_base_url: "".to_string(),
        }
    }
}

impl Settings {
    fn new() -> Self {
        let mut def_config = Config::default();
        def_config
            .merge(File::with_name("Settings"))
            .unwrap()
            .merge(Environment::with_prefix("APP"))
            .unwrap();

        Settings {
            sock_url_local: def_config.get_str("SOCKS_URL_LOCAL").unwrap(),
            socks_port_local: def_config.get("SOCKS_PORT_LOCAL").unwrap(),
            telegram_bot_id: def_config.get_str("TELEGRAM_BOT_ID").unwrap(),
            telegram_base_url: def_config.get_str("TELEGRAM_BASE_URL").unwrap(),
        }
    }
}
