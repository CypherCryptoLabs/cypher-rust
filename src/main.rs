#[macro_use]
mod debug;
mod config;
mod networking;
mod crypto;
mod blockchain;
mod transaction_queue;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() {
    config::load();
    crypto::init();
    transaction_queue::init();
    tokio::task::spawn_blocking(|| {networking::start_http_server()});
    println_debug!("Node listening to {}:1234", config::IP_ADDRESS.to_string());
    tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    
    let local_node = networking::node::Node::new(
        config::IP_ADDRESS.to_string(), 
        networking::node::LOCAL_BLOCKCHAIN_ADDRESS.to_string(), //dummy
        SystemTime::now().duration_since(UNIX_EPOCH)
            .expect("Time went backwards").as_millis() as u64
    );

    let seed_node = networking::node::Node::new(
        config::SEED_IP_ADDRESS.to_string(),
        config::SEED_WALLET_ADDRESS.to_string(),
        SystemTime::now().duration_since(UNIX_EPOCH)
            .expect("Time went backwards").as_millis() as u64
    );

    match local_node {
        Ok(_) => {},
        Err(e) => {
            println_debug!("{}", e);
            std::process::exit(1);
        },
    }

    match seed_node {
        Ok(_) => {},
        Err(e) => {
            println_debug!("{}", e);
            std::process::exit(1);
        },
    }

    let mut local_node_unwrapped = local_node.unwrap();
    let mut seed_node_unwrapped = seed_node.unwrap();
    seed_node_unwrapped.version = config::SEED_VERSION.to_string();

    unsafe {
        //let register_result = rt.block_on(local_node.unwrap().register());
        if !local_node_unwrapped.register().await {
            println_debug!("Could not add local Node to Node list.");
            std::process::exit(1);
        }
    }

    let registration_success =networking::register_to_network(&seed_node_unwrapped, &local_node_unwrapped).await;
    
    match registration_success {
        Ok(_) => {
            let registration_success_unwrapped = registration_success.unwrap();
            if !registration_success_unwrapped {
                println_debug!("Could not register Node to network!");
                std::process::exit(1);
            }

            let network_sync_success = networking::sync_node_list(&seed_node_unwrapped).await;
            if !network_sync_success {
                println_debug!("Could not sync network!");
                std::process::exit(1);
            }

        }
        Err(e) => {
            println_debug!("{}", e);
            std::process::exit(1);
        }
    }

}