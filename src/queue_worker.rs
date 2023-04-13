use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use crypto_hash::{hex_digest, Algorithm};
use num_bigint::BigUint;
use num_traits::{Zero, Num};

use crate::blockchain;
use crate::networking::node::{self, Node};

use super::transaction_queue;

pub fn init() {
    thread::spawn(|| {
        loop {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_millis() as u64;
            let next_voting_slot = now - (now % 60000) + 60000;
            let sleep_timer = next_voting_slot - now;

            thread::sleep(Duration::from_millis(sleep_timer));

            let forger = match select_forger(next_voting_slot) {
                Some(forger) => forger,
                None => continue,
            };

            let transactions_for_block: Vec<blockchain::Tx>;
            unsafe { 
                let mut transactions: Vec<blockchain::Tx> = transaction_queue::TX_HASHMAP.as_mut().unwrap().values().cloned().collect();
                transactions.sort_unstable_by(|a, b| b.network_fee.cmp(&a.network_fee));
                transactions_for_block= transactions.into_iter().take(100).collect();
                println_debug!("{:#?}", transactions_for_block);
            };

            let block = blockchain::Block::new(transactions_for_block);
            if block.timestamp == 0 {
                println_debug!("Block could not be created, skipping current iteration!");
                continue;
            }

            println_debug!("{:#?}", block);
        }
    });
}

fn select_forger(next_voting_slot: u64) -> Option<Node> {
    let node_list_copy = unsafe { node::NODE_LIST.clone() };
    let mut node_hashmap: HashMap<String, String> = HashMap::new();
    let forger: node::Node;
    let node_address_hashes_vec: Vec<&String>;

    let target_por = hex_digest(Algorithm::SHA256, format!("{}{}", next_voting_slot.to_string(), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855").as_bytes()); //hard coded for now, will be replaces by previous block hash

    for node in node_list_copy.clone() {
        let hash = hex_digest(Algorithm::SHA256, node.blockchain_address.as_bytes());
        node_hashmap.insert(hash, node.blockchain_address);
    }

    node_address_hashes_vec= node_hashmap.keys().collect();
    let target_biguint = BigUint::from_str_radix(&target_por, 16).unwrap();

    let forger_hash = match node_address_hashes_vec
        .iter()
        .min_by_key(|key| {
            let key_biguint = BigUint::from_str_radix(key, 16).unwrap();
            let diff = if key_biguint > target_biguint {
                key_biguint - target_biguint.clone()
            } else {
                target_biguint.clone() - key_biguint
            };
            diff
        }) {
            Some(key) => {key},
            None => {
                return None;
            },
        };
    
    let forger_address = node_hashmap.get(forger_hash.clone()).unwrap().to_owned();

    forger = match node_list_copy.iter().find(|s| s.blockchain_address == forger_address) {
        Some(node) => node.to_owned(),
        None => {return None;},
    };

    println_debug!("Forger: {:#?}\nTarget POR: {}\nPool: {:#?}", forger, target_por, node_hashmap);

    return Some(forger);
}