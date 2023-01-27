mod config;
mod networking;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() {
    config::load();
    
    let local_node = networking::node::Node::new(config::IP_ADDRESS.to_string(), 
    networking::node::LOCAL_BLOCKCHAIN_ADDRESS.to_string(), //dummy
        SystemTime::now().duration_since(UNIX_EPOCH)
            .expect("Time went backwards").as_micros() as u64
    );

    tokio::task::spawn_blocking(|| {networking::start_http_server()});
    println!("Node listening to {}:1234", config::IP_ADDRESS.to_string());
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    match local_node {
        Ok(_) => {
            unsafe {
                //let register_result = rt.block_on(local_node.unwrap().register());
                if !local_node.unwrap().register().await {
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
}