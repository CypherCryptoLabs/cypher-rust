extern crate serde;
extern crate serde_json;

use regex::Regex;
use serde::{Serialize, Deserialize};
use std::{time::{SystemTime, UNIX_EPOCH}};

pub static mut NODE_LIST: Vec<Node> = vec![];
pub static LOCAL_BLOCKCHAIN_ADDRESS: &str = "0x742d35Cc6634C0532925a3b844Bc454e4438f44e";

#[derive(Serialize, Deserialize, Clone)]
pub struct Node {
    ip_address: String,
    blockchain_address: String,
    registration_timestamp: u64
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NodeInfo {
    node_name: String,
    node_version: String,
    blockchain_address: String,
    unix_time: u64
}

impl Node {
    pub fn new(ip_address: String, blockchain_address: String, 
        registration_timestamp: u64) -> Result<Node, std::io::Error> {
            let now = SystemTime::now().duration_since(UNIX_EPOCH)
                .expect("Time went backwards").as_micros() as u64;

            if Regex::new(r"^(([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])\.){3}([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])$")
                .unwrap()
                .is_match(&ip_address) && 
                Regex::new(r"^0x[a-fA-F0-9]{40}$").unwrap().is_match(&blockchain_address) &&
                registration_timestamp <= now &&
                registration_timestamp > now - 10000
            {
                return Ok(
                    Node {
                        ip_address,
                        blockchain_address,
                        registration_timestamp
                    }
                )
            }

            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Could not create Node: Malformed data"))

    }

    pub async unsafe fn register(&self) -> bool {
        if !self.to_owned().is_reachable().await {
            return false;
        }

        let ip_already_in_use = NODE_LIST.iter().any(|n| n.ip_address == self.ip_address);
        let blockchain_address_already_in_use = NODE_LIST.iter().any(|n| n.blockchain_address == self.blockchain_address);

        if !ip_already_in_use && !blockchain_address_already_in_use {
            NODE_LIST.push(self.to_owned());
        }

        return !ip_already_in_use && !blockchain_address_already_in_use;
    }

    pub async fn is_reachable(self) -> bool {
        
        let body_string = super::client::http_request_timeout(self.ip_address).await;
        let body_string_result: String;

        match body_string {
            Ok(_) => {
                body_string_result = body_string.unwrap()
            }
            Err(_) => {
                println!("{:#?}", body_string.err());
                return false;
            }
        }

        let body_json_result:Result<NodeInfo, serde_json::Error> = serde_json::from_str(body_string_result.as_str());

        match body_json_result {
            Ok(_) => {
                let body_json = body_json_result.unwrap();
                let now = SystemTime::now().duration_since(UNIX_EPOCH)
                    .expect("Time went backwards").as_micros() as u64;
                if body_json.unix_time <= now && body_json.unix_time > now - 10000 
                    && body_json.blockchain_address == self.blockchain_address{
                    true
                } else {
                    println!("{:#?}", body_string_result);
                    false
                }
            }
            Err(_) => {
                println!("{:#?}",  body_json_result.err());
                false
            }
        }

    }
}