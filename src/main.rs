mod config;
mod consensus;
mod networking;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    config::load();
    
    println!("Node listening to {}:1234", config::IP_ADDRESS.to_string());
    
    let local_node = consensus::Node::new(config::IP_ADDRESS.to_string(), 
        "0x00".to_string(), 
        SystemTime::now().duration_since(UNIX_EPOCH)
            .expect("Time went backwards").as_micros()
    );

    local_node.register();

    networking::start_http_server();
}