use std::collections::HashMap;

use super::blockchain::Tx;

pub static mut TX_HASHMAP: Option<HashMap<String, Tx>> = None;

pub fn init() {
    // make the tx_hashmap actually a hashmap
    unsafe {TX_HASHMAP = Some(HashMap::new());}
}

pub fn insert(tx: &Tx) -> std::option::Option<Tx> {
    return unsafe {TX_HASHMAP.as_mut().unwrap().insert(tx.signature.clone(), tx.to_owned())};
}

pub fn get(signature: &String) -> std::option::Option<&'static Tx> {
    return unsafe {TX_HASHMAP.as_mut().unwrap().get(signature)};
}