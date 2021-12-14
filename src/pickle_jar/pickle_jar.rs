use tokio::sync::Mutex;
use std::sync::{Arc};
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use std::path::Path;
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::fmt;
use log::{info,error};

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
#[serde(tag = "row")]
pub struct Row {
    pub telegram_chat_id: i64,
    pub node_url: String,
    pub is_watching: bool,
    pub macaroon: String,
}

impl Default for Row {
    fn default () -> Row {
        return Row {
                telegram_chat_id:i64::MIN, 
                node_url: "".to_string(),
                is_watching: false,
                macaroon: "".to_string()
            };
    }
}

impl fmt::Display for Row {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return write!(f, r#"(
            'telegram_chat_id': '{}',
            'node_url': '{}',
            'is_watching': '{}',
            'macaroon': '{}'
        )"#, self.telegram_chat_id, self.node_url, self.is_watching, self.macaroon);
    }
}

#[derive(Clone)]
pub struct PickleJar {
    pub db: Arc<Mutex<PickleDb>>
}

impl PickleJar {
    pub fn init() -> Self {
        let path = build_path_to_pickle();
        let db_mutex;
        if !Path::new(&path[0..path.len()]).is_file() {
            db_mutex = Mutex::new(PickleDb::load(
                path,
                PickleDbDumpPolicy::AutoDump,
                SerializationMethod::Cbor,
            ).unwrap());
        } else {
            db_mutex = Mutex::new(PickleDb::load(
                path,
                PickleDbDumpPolicy::AutoDump,
                SerializationMethod::Cbor,
            ).unwrap());
        }
        Self { db: Arc::new(db_mutex) }
    }
    
    pub fn new(clone_db: Arc<Mutex<PickleDb>>) -> Self {
        Self{ db: clone_db}
    }

    pub async fn get(self, telegram_user_id: &i64) -> Row {
        let guard = self.db.lock()
                           .await;
        let unlock_db = & *guard;
 
        match unlock_db.get::<Row>(&telegram_user_id.to_string()) {
            Some(value) => {
                info!("{}'s amount is {}", telegram_user_id, value);
               return value;
            }
            None => {
                info!("No entry found");
                return Row::default();
            }
         }
    }
     
    pub async fn add(self, telegram_client_id: &str, row: Row) {
        let mut guard = self.db.lock()
                               .await;
    
        let unlock_db = &mut *guard;
        
        unlock_db.ladd(telegram_client_id, &row);
    }

    pub async fn remove(self, telegram_user_id: &str) -> bool {
        let mut guard = self.db.lock()
                            .await;
        
        let unlock_db = &mut *guard;

        match unlock_db.rem(telegram_user_id) {
            Ok(v) => {
                if v {
                    info!("{} has been successfully removed", telegram_user_id);
                    return true;
                } else {
                    info!("Record does not exist");
                    return false;
                }
            }
            Err(e) => {
                error!("Error, failed due to: {}", e);
                return false;
            }
        };
    }

    pub async fn get_values(self) -> Vec<Row> {
        let guard = self.db.lock()
                        .await;
        let unlock_db = & *guard;
        let mut values_vec:Vec<Row> = vec![];

        for kv in unlock_db.iter(){
            match kv.get_value() {
                Some(value) => { values_vec.push(value); }
                None => { }

            } 
        }

        return values_vec;
    }
}

fn build_path_to_pickle()-> String{
    let root = config_dir().unwrap();
    let the_way = root.to_str().unwrap();
    let true_way = the_way.to_owned();
    let the_path = true_way + "/.pickleDb";
    return the_path;
}