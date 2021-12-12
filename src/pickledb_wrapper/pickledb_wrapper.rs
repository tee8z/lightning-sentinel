use tokio::sync::Mutex;
use std::sync::{Arc};
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use std::path::Path;
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone)]
pub struct Pickle {
    db: Arc<Mutex<PickleDb>>
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
#[serde(tag = "row")]
pub struct Row {
    telegram_chat_id: i64,
    node_url: String,
    is_watching: bool,
    macaroon: String,
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
            'telegram_chat_id': '{}'
            'node_url': {}
            'is_watching': {}
            'macaroon': {}
        )"#, self.telegram_chat_id, self.node_url, self.is_watching, self.macaroon);
    }
}

impl Default for Pickle {
    fn default() -> Pickle {
        let the_path = build_path_to_pickle();
        return Pickle {
            db: Arc::new(Mutex::new(PickleDb::new(
                the_path,
                PickleDbDumpPolicy::AutoDump,
                SerializationMethod::Cbor,
            )))
        }
    }
}


impl Pickle {
    pub fn new() -> Self {
        let mut pickle = Pickle::default();
        let the_path = build_path_to_pickle();
        if !Path::new(&the_path[0..the_path.len()]).is_file() {
            return pickle;
        } else {
            let db_mutex = Mutex::new(PickleDb::load(
                the_path,
                PickleDbDumpPolicy::AutoDump,
                SerializationMethod::Cbor,
            ).unwrap());
            pickle.db = Arc::new(db_mutex);
        }
        return pickle;
    }

    pub async fn get(self, telegram_user_id: &i64) -> Row {
        let guard = self.db.lock()
                           .await;
        let unlock_db = & *guard;
 
        match unlock_db.get::<Row>(&telegram_user_id.to_string()) {
            Some(value) => {
                println!("{}'s amount is {}", telegram_user_id, value);
               return value;
            }
            None => {
                println!("No entry found");
                return Row::default();
            }
         }
    }

    pub async fn remove(self, telegram_user_id: &str) -> bool {
        let mut guard = self.db.lock()
                            .await;
        
        let mut unlock_db = &mut *guard;

        match unlock_db.rem(telegram_user_id) {
            Ok(v) => {
                if v {
                    println!("{} has been successfully removed", telegram_user_id);
                    return true;
                } else {
                    println!("Record does not exist");
                    return false;
                }
            }
            Err(e) => {
                println!("Error, failed due to: {}", e);
                return false;
            }
        };
    }

    pub async fn get_keys(self) -> Vec<String> {
        let guard = self.db.lock()
                        .await;
        let unlock_db = & *guard;
        return unlock_db.get_all();
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