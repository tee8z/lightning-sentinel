use tokio::sync::mpsc::{channel, Receiver, Sender};

use std::fmt;

// NOTE: user_id will be unique for each user connecting to the bot

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct ChannelMessage {
    pub channel_type: ChannelType,
    pub user_id: i64,
    pub node_url: String,
    pub command: String,
    pub message: String,
    pub macaroon: String,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum ChannelType {
    Ln = 0x00,
    Tel = 0x01,
    Def = 0x02,
}

impl Default for ChannelMessage {
    fn default() -> ChannelMessage {
        ChannelMessage {
            channel_type: ChannelType::Def,
            user_id: i64::MIN,
            node_url: "".to_string(),
            command: "".to_string(),
            message: "".to_string(),
            macaroon: "".to_string(),
        }
    }
}

impl fmt::Display for ChannelType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return write!(f, "{:?}", self);
    }
}

impl fmt::Display for ChannelMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return write!(
            f,
            r#"(
            'channel_type': '{}',
            'chat_id': '{}',
            'node_url':'{}',
            'command': '{}',
            'message': '{}',
            'macaroon': '{}'
        )"#,
            self.channel_type,
            self.user_id,
            self.node_url,
            self.command,
            self.message,
            self.macaroon
        );
    }
}
//mpsc -> A multi-producer, single-consumer queue for sending values between asynchronous tasks.
pub struct Messages {
    pub nostr_messages: (Sender<ChannelMessage>, Receiver<ChannelMessage>),
    pub lightning_messages: (Sender<ChannelMessage>, Receiver<ChannelMessage>),
}

//Queue up to 100 messages, this will keep the bot from over loading either API, force them to go one at a time (may become a bottle neck down the road)
impl Messages {
    pub fn new() -> Self {
        Messages {
            nostr_messages: channel(100),
            lightning_messages: channel(100),
        }
    }
}
