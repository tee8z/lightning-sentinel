use nostr_bot::*;


struct WatchNode {
    nostr_pubkey: String,
    node_url: String,
    macaroon: String,
}

struct StopWatchNode {
    nostr_pubkey: String,
    node_url: String,
}

struct NostrBot {}

impl NostrBot {
    pub fn new(&self) -> NostrBot {
        Bot::new(key)
    }
}