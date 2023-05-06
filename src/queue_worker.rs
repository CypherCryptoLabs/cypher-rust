use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use crypto_hash::{hex_digest, Algorithm};
use num_bigint::BigUint;
use num_traits::Num;
use tokio::runtime::Runtime;

use crate::blockchain::{self, Vouch, Block};
use crate::networking::node::{self, Node};
use crate::networking::route_handler;

use super::transaction_queue;

pub static mut CURRENT_PROPOSED_BLOCK: Block = Block { 
    timestamp: 0, 
    parent_block_hash: String::new(), 
    forger: String::new(), 
    payload: vec![], 
    forger_signature: String::new(), 
    forger_pub_key: String::new(),
    validators: vec![] 
};

pub static mut CURRENT_PROPOSED_BLOCK_VOUCHES: Vec<Vouch> = vec![];

pub fn init() {
    thread::spawn(|| {
        let rt = Runtime::new().unwrap();
        loop {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_millis() as u64;
            let next_voting_slot = now - (now % 60000) + 60000;
            let sleep_timer = next_voting_slot - now;

            thread::sleep(Duration::from_millis(sleep_timer));
            unsafe { CURRENT_PROPOSED_BLOCK = Block { 
                timestamp: 0, 
                parent_block_hash: String::new(), 
                forger: String::new(), 
                payload: vec![], 
                forger_signature: String::new(),
                forger_pub_key: String::new(),
                validators: vec![] 
            }};

            let node_list_copy = unsafe { node::NODE_LIST.clone() };
            let forger = match select_forger(next_voting_slot, node_list_copy.clone()) {
                Some(forger) => forger,
                None => continue,
            };
            let validators = select_validators(node_list_copy, forger.blockchain_address.clone());
            let num_validators = validators.len();
            let validator_addresses = validators.iter().map(|node| node.blockchain_address.clone()).collect();

            println_debug!("Forger: {:#?}\nValidators:{:#?}", forger, validators);
            unsafe { 
                super::networking::CURRENT_FORGER_ADDRESS = forger.blockchain_address.clone();
                super::networking::CURRENT_VALIDATORS = validators.clone();
            };

            if super::networking::node::LOCAL_BLOCKCHAIN_ADDRESS.to_string() == forger.blockchain_address {
                println_debug!("This node is the forger for the current slot!");

                let transactions_for_block: Vec<blockchain::Tx>;
                unsafe { 
                    let mut transactions: Vec<blockchain::Tx> = transaction_queue::TX_HASHMAP.as_mut().unwrap().values().cloned().collect();
                    transactions.sort_unstable_by(|a, b| b.network_fee.cmp(&a.network_fee));
                    transactions_for_block= transactions.into_iter().take(100).collect();
                    println_debug!("{:#?}", transactions_for_block);
                };

                unsafe { 
                    CURRENT_PROPOSED_BLOCK = blockchain::Block::new(transactions_for_block, validator_addresses) ;
                    if CURRENT_PROPOSED_BLOCK.timestamp == 0 {
                        println_debug!("Block could not be created, skipping current iteration!");
                        continue;
                    }

                    let cloned_block = CURRENT_PROPOSED_BLOCK.clone();
                    rt.spawn(async move {
                        cloned_block.broadcast_to_validators(validators).await;
                    });

                    println_debug!("{:#?}", CURRENT_PROPOSED_BLOCK);
                }

            } else if validators.iter().any(|n| n.blockchain_address == super::networking::node::LOCAL_BLOCKCHAIN_ADDRESS.to_string()) {
                println_debug!("This node is a validator for the current slot!");
                // wait 5 seconds, every validator should receive a copy of the proposed Block
                thread::sleep(Duration::from_millis(5000));

                // everyone should have received a copy of the proposed Block by now
                if unsafe { CURRENT_PROPOSED_BLOCK.timestamp } == 0 {
                    println_debug!("Expected to receive Block, but received none (Invalid?). Skipping vouching/voting");
                    continue;
                }

                if !unsafe { CURRENT_PROPOSED_BLOCK.is_valid(&forger, validator_addresses) } {
                    println_debug!("Proposed Block was invalid. Skipping vouching/voting.");
                    continue;
                }

                let vouch = unsafe { CURRENT_PROPOSED_BLOCK.vouch() };
                // share vouch with other validators
                
                rt.spawn(async move {
                    vouch.broadcast_to_validators(validators).await;
                });

                thread::sleep(Duration::from_millis(
                    next_voting_slot + 
                    15000 - 
                    SystemTime::now().duration_since(UNIX_EPOCH)
                        .expect("Time went backwards")
                        .as_millis() as u64)
                );

                unsafe {
                    CURRENT_PROPOSED_BLOCK_VOUCHES.iter().for_each(|vouch| {
                        CURRENT_PROPOSED_BLOCK.validators.iter_mut().for_each(|mut block_vouch| {
                            if block_vouch.address == vouch.address {
                                block_vouch.pub_key = vouch.pub_key.clone();
                                block_vouch.signature = vouch.signature.clone();
                            }
                        })
                    });

                    if CURRENT_PROPOSED_BLOCK_VOUCHES.len() <= num_validators / 2 {
                        continue;
                    }

                    let metadata_wrapped_block = route_handler::MetaData::new(CURRENT_PROPOSED_BLOCK.clone());
                    rt.spawn(async move {
                        metadata_wrapped_block.broadcast("/blockchain/block".to_string()).await;
                    });

                }
                
            } else {
                println_debug!("This node is inactive for the current slot!");
            }
        }
    });
    
}

fn select_forger(next_voting_slot: u64 , node_list_copy: Vec<Node>) -> Option<Node> {
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

    return Some(forger);
}

fn select_validators(mut node_list: Vec<Node>, forger_address: String) -> Vec<Node>{
    let max_validators = 128; // the maximum number of validators for each slot
    let max_available_validators = node_list.len();

    if max_available_validators < max_validators {
        // return the node list without the forger
        node_list.retain(|x| x.blockchain_address != forger_address);
        return node_list;
    } else {
        // select validators
        todo!()
    }
}