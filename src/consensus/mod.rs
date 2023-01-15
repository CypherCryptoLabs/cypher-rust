pub struct Node {
    pub ip_address: String,
    pub blockchain_address: String,
    pub registration_timestamp: u128
}

impl Node {
    pub fn new(ip_address: String, blockchain_address: String, 
        registration_timestamp: u128) -> Node {
        Node {
            ip_address,
            blockchain_address,
            registration_timestamp
        }
    }

    pub fn register(&self) {
        // todo: add Node to some array, which holds all known Nodes in the
        // network
    }
}