
use tokio::sync::mpsc::{channel, Sender, Receiver};
use std::fmt;

pub struct LnInfo {
    pub node_url:String,
    pub command:String,
    pub is_active:bool,
    pub message:String,
}

impl fmt::Display for LnInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return write!(f, "Node_URL: {}\n\rIsActive: {}\n\rCommand: {},\n\rMessage: {} ", self.node_url, self.is_active, self.command, self.message);
    }
}

pub struct TelInfo {
    pub user_id:String,
    pub command:String,
    pub is_active:bool,
    pub message:String,
}

impl fmt::Display for TelInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return write!(f, "User_Id: {}\n\rIsActive: {}\n\rCommand: {},\n\rMessage: {} ", self.user_id, self.is_active, self.command, self.message);
    }
}
//mpsc -> A multi-producer, single-consumer queue for sending values between asynchronous tasks.
pub struct Messages { 
    pub telegram_messages:(Sender<TelInfo>, Receiver<TelInfo>),
    pub lightning_messages:(Sender<LnInfo>, Receiver<LnInfo>)
}


//Queue up to 100 messages, this will keep from me over loading either API, force them to go one at a time (may become a bottle neck down the road)
impl Messages {
    pub fn new() -> Self {
        let messages = Messages {
            telegram_messages:channel(100),
            lightning_messages:channel(100),
        };
       return messages;
    }
}
