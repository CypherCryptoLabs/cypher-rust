use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetInfo {
    pub node_name: String,
    pub node_version: String,
    pub unix_time: u64,
    pub blockchain_address: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GetNodes {
    pub nodes: Vec<super::super::node::Node>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PostNode {
    pub status: bool
}