use std::time::{SystemTime, UNIX_EPOCH};
use node::Node;
use regex::Regex;
use crate::networking::node::LOCAL_BLOCKCHAIN_ADDRESS;
use crate::{networking, crypto};
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

impl Tx {
    pub fn is_valid(&self) -> bool{
        // check if receiver address is valid
        if !Regex::new(r"^[13][a-km-zA-HJ-NP-Z1-9]{25,34}$")
                .unwrap()
                .is_match(&self.receiver_address)
        {
            return false;
        }

        // check if sendder pub key is valid
        let secp_pub_key = match super::crypto::string_to_pub_key(&self.sender_pub_key) {
            Some(key) => key,
            None => {return false;}
        };
        let sender_address = super::crypto::pub_key_to_address(&secp_pub_key);

        if !Regex::new(r"^[13][a-km-zA-HJ-NP-Z1-9]{25,34}$")
                .unwrap()
                .is_match(&sender_address)
        {
            return false
        }

        // TODO: implement a balance check for sender

        // check if signature is valid

        let mut tx_cpy = self.clone();
        tx_cpy.signature = "".to_string();

        let tx_string = match serde_json::to_string(&tx_cpy) {
            Ok(string) => string,
            Err(e) => {
                println_debug!("{:#?}", e);
                return false;
            }
        };

        let signature_is_valid = super::crypto::verify_signature(self.signature.as_str(), tx_string.as_str(), &self.sender_pub_key);
        println_debug!("Signature is valid: {}", signature_is_valid);

        return true;
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Block {
    pub timestamp: u64,
    pub parent_block_hash: String,
    pub forger: String,
    pub payload: Vec<Tx>,
    pub forger_signature: String,
    pub forger_pub_key: String,
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
            forger_pub_key: unsafe {super::crypto::PUBLIC_KEY.unwrap().to_string()},
            validators: vec![],
            parent_block_hash: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),
        };

        validators.iter().for_each(|address| {
            temp_block.validators.push(Vouch::new(address, &String::new()));
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

    pub fn is_valid(&self, forger: &Node, validators: Vec<String>) -> bool {
        // check if previous Block Hash is correct (hardcoded until Block storage is implemented)
        if self.parent_block_hash != "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string() {
            println_debug!("1");
            return false;
        }

        // check if ty are valid
        if !self.payload.iter().all(|tx| tx.is_valid()) {
            println_debug!("2");
            return false;
        }

        // check if forger and forger signature are valid
        if self.forger != forger.blockchain_address {
            println_debug!("3");
            return false;
        }

        let mut block_cpy = self.clone();
        block_cpy.forger_signature = "".to_string();

        let block_json = match serde_json::to_string(&block_cpy.clone()) {
            Ok(json) => {
                json
            },
            Err(e) => {
                println_debug!("{}", e);
                println_debug!("4");
                return false;
            }
        };

        if !crypto::verify_signature(self.forger_signature.as_str(), block_json.as_str(), &self.forger_pub_key) {
            println_debug!("5");
            return false;
        }

        let forger_pub_key = match super::crypto::string_to_pub_key(&self.forger_pub_key) {
            Some(pub_key) => pub_key,
            None => {
                println_debug!("6");
                return false;
            }
        };
        let forger_expected_address = super::crypto::pub_key_to_address(&forger_pub_key);

        if self.forger != forger_expected_address {
            println_debug!("7");
            println_debug!("{}  {}", self.forger, forger_expected_address);
            return false;
        }

        if !validators.iter().all(|address| {
            self.validators.iter().any(|vouch| address.to_owned() == vouch.address)
        }) {
            println_debug!("8");
            return false;
        }

        return true;
    }

    pub fn vouch(&self) -> Vouch {
        let mut block_clone = self.clone();
        block_clone.validators.iter_mut().for_each(|vouch| {
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

        let mut vouch = Vouch::new(&LOCAL_BLOCKCHAIN_ADDRESS, unsafe { &crypto::PUBLIC_KEY.unwrap().to_string() });
        vouch.signature = crypto::sign_string(&block_json);

        return vouch;
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]

pub struct Vouch {
    pub address: String,
    pub signature: String,
    pub pub_key: String
}

impl Vouch {
    pub fn new(address: &String, pub_key: &String) -> Vouch {
        let vouch = Vouch {
            address: address.to_string(),
            signature: String::new(),
            pub_key: pub_key.to_string(),
        };

        return vouch;
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
                "/v".to_string() + &node.version + "/blockchain/vouch",
                json_metadata_block.to_string(),
            ).await;

            println_debug!("{:#?}", result);
        }
    }

    pub fn is_valid(&self, block: Block) -> bool {
        if !block.validators.iter().any(|validator| validator.address == self.address) { 
            return false;
        };

        let pub_key = match super::crypto::string_to_pub_key(&self.pub_key) {
            Some(pub_key) => pub_key,
            None => {
                println_debug!("6");
                return false;
            }
        };

        let expected_address = super::crypto::pub_key_to_address(&pub_key);

        if self.address != expected_address {
            return false;
        }

        return true;
    }
}