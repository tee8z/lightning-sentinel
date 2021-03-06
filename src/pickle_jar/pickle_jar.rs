use dirs::config_dir;
use log::{error, info};
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
#[serde(tag = "row")]
pub struct Row {
    pub telegram_chat_id: i64,
    pub node_url: String,
    pub is_watching: bool,
    pub macaroon: String,
}

impl Default for Row {
    fn default() -> Row {
        Row {
            telegram_chat_id: i64::MIN,
            node_url: "".to_string(),
            is_watching: false,
            macaroon: "".to_string(),
        }
    }
}

impl fmt::Display for Row {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return write!(
            f,
            r#"(
            'telegram_chat_id': '{}',
            'node_url': '{}',
            'is_watching': '{}',
            'macaroon': '{}'
        )"#,
            self.telegram_chat_id, self.node_url, self.is_watching, self.macaroon
        );
    }
}

#[derive(Clone)]
pub struct PickleJar {
    pub db: Arc<Mutex<PickleDb>>,
}

impl PickleJar {
    pub fn init() -> Self {
        let path = build_path_to_pickle();
        info!("(init) path: {}", path);
        let pickle;
        if !Path::new(&path[0..path.len()]).is_file() {
            pickle = PickleDb::new(
                path,
                PickleDbDumpPolicy::AutoDump,
                SerializationMethod::Cbor,
            );
        } else {
            match PickleDb::load(
                path.clone(),
                PickleDbDumpPolicy::AutoDump,
                SerializationMethod::Cbor,
            ){
                Ok(found_pickle) => {
                    pickle = found_pickle
                }
                Err(err) => {
                    info!("(init) Error in loading existing pickle, creating a new one {:#?}", err);
                    pickle = PickleDb::new(
                        path,
                        PickleDbDumpPolicy::AutoDump,
                        SerializationMethod::Cbor,
                    );
                }
            }
        }
        Self {
            db: Arc::new(Mutex::new(pickle)),
        }
    }

    pub fn new(clone_db: Arc<Mutex<PickleDb>>) -> Self {
        Self { db: clone_db }
    }

    pub async fn get(self, telegram_user_id: &i64) -> Row {
        let guard = self.db.lock().await;
        let unlock_db = &*guard;

        match unlock_db.get::<Row>(&telegram_user_id.to_string()) {
            Some(value) => {
                value
            }
            None => {
                Row::default()
            }
        }
    }

    pub async fn set(self, telegram_client_id: &str, row: Row) {
        let mut guard = self.db.lock().await;

        let unlock_db = &mut *guard;
        match unlock_db.set(telegram_client_id, &row) {
            Ok(_) => {}
            Err(e) => {
                error!("(set) {}", e);
            }
        }
    }

    pub async fn remove(self, telegram_user_id: &i64) -> bool {
        let mut guard = self.db.lock().await;

        let unlock_db = &mut *guard;

        match unlock_db.rem(&telegram_user_id.to_string()) {
            Ok(v) => {
                return v;
            }
            Err(e) => {
                error!("(remove) error: {}", e);
            }
        };
        false
    }

    pub async fn get_values(self) -> Vec<Row> {
        let guard = self.db.lock().await;
        let unlock_db = &*guard;
        let mut values_vec: Vec<Row> = vec![];

        for kv in unlock_db.iter() {
            if let Some(value) = kv.get_value() {
                    values_vec.push(value);
            }
        }

        values_vec
    }
}

fn build_path_to_pickle() -> String {
    let root = config_dir().unwrap();
    let the_way = root.to_str().unwrap();
    let true_way = the_way.to_owned();
    true_way + "/.pickleDb"
}
