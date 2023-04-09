use std::time::{SystemTime, UNIX_EPOCH};
use crate::networking::node::LOCAL_BLOCKCHAIN_ADDRESS;

use super::networking::node;

extern crate serde;
extern crate serde_json;
extern crate rand;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Tx {
    pub amount: u64,
    pub network_fee: u64,
    pub sender_pub_key: String,
    pub receiver_address: String,
    pub timestamp: u64,
    pub signature: String
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Block {
    pub timestamp: u64,
    pub forger: String,
    pub payload: Vec<Tx>,
    pub forger_signature: String,
    pub validators: Vec<(String, String)> 
}

impl Block {
    pub fn new(tx: Vec<Tx>) -> Block {
        let mut temp_block =  Block {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)
            .expect("Time went backwards").as_millis() as u64,
            forger: LOCAL_BLOCKCHAIN_ADDRESS.to_string(),
            payload: tx,
            forger_signature: "".to_string(),
            validators: vec![],
        };

        let block_json = match serde_json::to_string(&temp_block.clone()) {
            Ok(json) => {
                json
            },
            Err(e) => {
                println_debug!("{}", e);
                temp_block.timestamp = 0;
                "".to_string()
            }
        };

        temp_block.forger_signature = super::crypto::sign_string(&block_json);
        return temp_block;
        
    }
}