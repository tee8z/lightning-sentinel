
use tokio::sync::mpsc::{channel, Sender, Receiver};
use std::fmt;

// NOTE: Chat_id will be unique for each user connecting to the bot
// https://stackoverflow.com/questions/59748008/telegram-bot-api-is-the-chat-id-unique-for-each-user-contacting-the-bot

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct ChannelMessage {
    pub channel_type:ChannelType,
    pub chat_id:i64,
    pub node_url:String,
    pub command:String,
    pub message:String,
    pub macaroon:String
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum ChannelType {
    LN = 0x00,
    TEL = 0x01
}

impl fmt::Display for ChannelType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return write!(f, "{:?}", self);
    }
}

impl fmt::Display for ChannelMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       
        return write!(f, r#"(
            'channel_type': '{}',
            'chat_id': '{}',
            'node_url':'{}',
            'command': '{}',
            'message': '{}',
            'macaroon': '{}'
        )"#, self.channel_type, self.chat_id, self.node_url, self.command, self.message, self.macaroon);
    }
}
//mpsc -> A multi-producer, single-consumer queue for sending values between asynchronous tasks.
pub struct Messages { 
    pub telegram_messages:(Sender<ChannelMessage>, Receiver<ChannelMessage>),
    pub lightning_messages:(Sender<ChannelMessage>, Receiver<ChannelMessage>)
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
