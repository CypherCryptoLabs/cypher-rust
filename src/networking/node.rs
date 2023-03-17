extern crate serde;
extern crate serde_json;
extern crate rand;

use regex::Regex;
use rand::Rng;
use serde::{Serialize, Deserialize};
use std::{time::{SystemTime, UNIX_EPOCH}};

pub static mut NODE_LIST: Vec<Node> = vec![];
pub static LOCAL_BLOCKCHAIN_ADDRESS: once_cell::sync::Lazy<String> = once_cell::sync::Lazy::new(|| {
    unsafe {
        super::super::crypto::BLOCKCHAIN_ADDRESS.clone()
    }
    
});

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Node {
    pub ip_address: String,
    pub blockchain_address: String,
    pub registration_timestamp: u64,
    pub version: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NodeInfo {
    pub node_name: String,
    pub node_version: String,
    pub blockchain_address: String,
    pub unix_time: u64
}

impl Node {
    pub fn new(ip_address: String, blockchain_address: String, 
        registration_timestamp: u64) -> Result<Node, std::io::Error> {
            let now = SystemTime::now().duration_since(UNIX_EPOCH)
                .expect("Time went backwards").as_millis() as u64;

            if Regex::new(r"^(([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])\.){3}([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])$")
                .unwrap()
                .is_match(&ip_address) && 
                Regex::new(r"^[13][a-km-zA-HJ-NP-Z1-9]{25,34}$").unwrap().is_match(&blockchain_address) &&
                registration_timestamp <= now + 1000 &&
                registration_timestamp > now - 10000
            {
                return Ok(
                    Node {
                        ip_address,
                        blockchain_address,
                        registration_timestamp,
                        version: "".to_string(),
                    }
                )
            }

            println_debug!("{:#?}", blockchain_address);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Could not create Node: Malformed data"))

    }

    pub async unsafe fn register(&mut self) -> bool {
        if !self.to_owned().is_reachable(self).await {
            return false;
        }

        let ip_already_in_use = NODE_LIST.iter().any(|n| n.ip_address == self.ip_address);
        let blockchain_address_already_in_use = NODE_LIST.iter().any(|n| n.blockchain_address == self.blockchain_address);

        if !ip_already_in_use && !blockchain_address_already_in_use {
            NODE_LIST.push(self.to_owned());
            
            tokio::spawn({

                let node_metadata = super::route_handler::MetaData::new(self.clone());
                node_metadata.broadcast()

                //node_clone.broadcast_registration()
            });

            return true;
        }

        return !ip_already_in_use && !blockchain_address_already_in_use;
    }

    pub async fn is_reachable(self, node_ref: &mut Node) -> bool {
        
        let body_string = super::client::http_get_request_timeout(self.ip_address, "/".to_string()).await;
        let body_string_result: String;

        match body_string {
            Ok(_) => {
                body_string_result = body_string.unwrap()
            }
            Err(_) => {
                println_debug!("{:#?}", body_string.err());
                return false;
            }
        }

        let body_json_result:Result<super::route_handler::MetaData<NodeInfo>, serde_json::Error> = serde_json::from_str(body_string_result.as_str());

        match body_json_result {
            Ok(_) => {
                let body_json = body_json_result.unwrap().payload;
                let now = SystemTime::now().duration_since(UNIX_EPOCH)
                    .expect("Time went backwards").as_millis() as u64;
                if body_json.unix_time <= now + 10000 && body_json.unix_time > now - 10000 
                    && body_json.blockchain_address == self.blockchain_address
                {
                    node_ref.version = body_json.node_version;
                    true
                } else {
                    println_debug!("{:#?} {:#?}", body_json.unix_time <= now + 10000 && body_json.unix_time > now - 10000, body_json.blockchain_address == self.blockchain_address);
                    println_debug!("Different data received, than expected during reachability check!");
                    false
                }
            }
            Err(_) => {
                println_debug!("{:#?}",  body_json_result.err());
                false
            }
        }

    }
}