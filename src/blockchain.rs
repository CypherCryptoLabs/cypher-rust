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