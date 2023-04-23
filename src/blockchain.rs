use std::time::{SystemTime, UNIX_EPOCH};
use node::Node;
use crate::networking::node::LOCAL_BLOCKCHAIN_ADDRESS;
use crate::networking;
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
    pub parent_block_hash: String,
    pub forger: String,
    pub payload: Vec<Tx>,
    pub forger_signature: String,
    pub validators: Vec<Vouch> 
}

impl Block {
    pub fn new(tx: Vec<Tx>, validators: Vec<String>) -> Block {
        println_debug!("{:#?}", validators);

        let mut temp_block =  Block {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)
            .expect("Time went backwards").as_millis() as u64,
            forger: LOCAL_BLOCKCHAIN_ADDRESS.to_string(),
            payload: tx,
            forger_signature: "".to_string(),
            validators: vec![],
            parent_block_hash: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),
        };

        validators.iter().for_each(|address| {
            temp_block.validators.push(Vouch::new(address));
        });

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

    pub async fn broadcast_to_validators(&self, validators: Vec<Node>) {
        let metadata_wrapped_block = unsafe { networking::route_handler::MetaData::new(self.to_owned()) };
        let json_metadata_block = match serde_json::to_string(&metadata_wrapped_block) {
            Ok(json) => json,
            Err(e) => {
                println_debug!("{:#?}", e);
                return;
            },
        };

        for node in validators {
            let result = networking::client::http_post_request_timeout(
                node.ip_address,
                "/v".to_string() + &node.version + "/blockchain/propose",
                json_metadata_block.to_string(),
            ).await;

            println_debug!("{:#?}", result);
        }
    }

    pub fn validate(&self) -> bool {
        return true;
    }

    pub fn vouch(&self) -> String {
        let mut block_clone = self.clone();
        block_clone.validators.iter().for_each(|vouch| {
            vouch.signature.to_owned();
            vouch.signature = "".to_string();
        });

        let block_json = match serde_json::to_string(&block_clone.clone()) {
            Ok(json) => {
                json
            },
            Err(e) => {
                println_debug!("{}", e);
                block_clone.timestamp = 0;
                "".to_string()
            }
        };

        return "".to_string();
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]

pub struct Vouch {
    address: String,
    signature: String
}

impl Vouch {
    pub fn new(address: &String) -> Vouch {
        let vouch = Vouch {
            address: address.to_string(),
            signature: "".to_string()
        };

        return vouch;
    }
}