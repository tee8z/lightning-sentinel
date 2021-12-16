use serde::{Deserialize, Serialize};
use std::{collections::HashMap};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LnGetInfo {
    pub version:String,
    pub commit_hash:String,
    pub identity_pubkey:String,
    pub alias:String,
    pub color:String,
    pub num_pending_channels:i64,
    pub num_active_channels:i64,
    pub num_inactive_channels:i64,
    pub num_peers:i64,
    pub block_height:i64,
    pub block_hash:String,
    pub best_header_timestamp:String,
    pub synced_to_chain:bool,
    pub synced_to_graph:bool,
    pub testnet: bool,
    pub chains: Vec<ChainData>,
    pub uris: Vec<String>,
    pub features: HashMap<i64, Feature>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChainData {
    pub chain:String,
    pub network:String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Feature {
    pub name: String,
    pub is_required: bool,
    pub is_known: bool
}

