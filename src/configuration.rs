
use config::{Config, File, Environment};

lazy_static::lazy_static! {
    pub static ref SETTINGS: Settings = Settings::new();
}


pub struct Settings {
    pub sock_url_local: String,
    pub socks_port_local: u16, 
    pub check_url: String,
    pub macaroon: String,
    pub telegram_bot_id: String,
    pub telegram_base_url: String,
}


impl Default for Settings {
    fn default() -> Self {
        Settings {
            sock_url_local: "socks5h://127.0.0.1".to_string(),
            socks_port_local: 9056,
            check_url: "".to_string(),
            macaroon: "".to_string(),
            telegram_bot_id: "".to_string(),
            telegram_base_url: "".to_string()
        }
    }
}



impl Settings{
    fn new() -> Self {
        let mut def_config = Config::default();
        def_config.merge(File::with_name("Settings.toml")).unwrap()
                 .merge(Environment::with_prefix("APP")).unwrap();
        let mut settings = Settings::default();
        settings.sock_url_local = def_config.get_str("SOCKS_URL_LOCAL").unwrap();
        settings.socks_port_local = def_config.get("SOCKS_PORT_LOCAL").unwrap();
        settings.check_url = def_config.get_str("CHECK_URL").unwrap();
        settings.macaroon = def_config.get_str("MACAROON").unwrap();
        settings.telegram_bot_id = def_config.get_str("TELEGRAM_BOT_ID").unwrap();
        settings.telegram_base_url = def_config.get_str("TELEGRAM_BASE_URL").unwrap();
        return settings;
    }

}
