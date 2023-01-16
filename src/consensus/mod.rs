use regex::Regex;
use serde::{Serialize, ser::SerializeStruct};
use std::{time::{SystemTime, UNIX_EPOCH}};

pub static mut NODE_LIST: Vec<Node> = vec![];

pub struct Node {
    pub ip_address: String,
    pub blockchain_address: String,
    pub registration_timestamp: u64
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

    pub unsafe fn register(self) -> bool {
        let ip_already_in_use = NODE_LIST.iter().any(|n| n.ip_address == self.ip_address);
        let blockchain_address_already_in_use = NODE_LIST.iter().any(|n| n.blockchain_address == self.blockchain_address);

        if !ip_already_in_use && !blockchain_address_already_in_use {
            NODE_LIST.push(self);
        }

        return !ip_already_in_use && !blockchain_address_already_in_use;
    }
}

impl Serialize for Node {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Node", 2)?;
        state.serialize_field("ip_address", &self.ip_address)?;
        state.serialize_field("blockchain_address", &self.blockchain_address)?;
        state.serialize_field("registration_timestamp", &self.registration_timestamp)?;
        // other fields
        state.end()
    }
}