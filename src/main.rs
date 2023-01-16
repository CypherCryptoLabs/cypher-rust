mod config;
mod consensus;
mod networking;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    config::load();
    
    println!("Node listening to {}:1234", config::IP_ADDRESS.to_string());
    
    let local_node = consensus::Node::new(config::IP_ADDRESS.to_string(), 
        "0x742d35Cc6634C0532925a3b844Bc454e4438f44e".to_string(), //dummy
        SystemTime::now().duration_since(UNIX_EPOCH)
            .expect("Time went backwards").as_micros() as u64
    );

    match local_node {
        Ok(_) => {
            unsafe {
                if !local_node.unwrap().register() {
                    println!("Could not add Node to Node list.");
                    std::process::exit(1);
                }
            }
        },
        Err(e) => {
            println!("{}", e);
            std::process::exit(1);
        },
    }
    networking::start_http_server();
}