mod config;
mod networking;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() {
    config::load();
    
    let local_node = networking::node::Node::new(
        config::IP_ADDRESS.to_string(), 
        networking::node::LOCAL_BLOCKCHAIN_ADDRESS.to_string(), //dummy
        SystemTime::now().duration_since(UNIX_EPOCH)
            .expect("Time went backwards").as_micros() as u64
    );

    let seed_node = networking::node::Node::new(
        config::SEED_IP_ADDRESS.to_string(),
        config::SEED_WALLET_ADDRESS.to_string(),
        SystemTime::now().duration_since(UNIX_EPOCH)
            .expect("Time went backwards").as_micros() as u64
    );

    match local_node {
        Ok(_) => {},
        Err(e) => {
            println!("{}", e);
            std::process::exit(1);
        },
    }

    match seed_node {
        Ok(_) => {},
        Err(e) => {
            println!("{}", e);
            std::process::exit(1);
        },
    }

    let local_node_unwrapped = local_node.unwrap();
    let seed_node_unwrapped = seed_node.unwrap();

    let registration_success =networking::register_to_network(&seed_node_unwrapped, &local_node_unwrapped).await;
    
    match registration_success {
        Ok(_) => {
            let registration_success_unwrapped = registration_success.unwrap();
            if !registration_success_unwrapped {
                println!("Could not register Node to network!");
                std::process::exit(1);
            }
        }
        Err(e) => {
            println!("{}", e);
            std::process::exit(1);
        }
    }

    tokio::task::spawn_blocking(|| {networking::start_http_server()});
    println!("Node listening to {}:1234", config::IP_ADDRESS.to_string());
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    unsafe {
        //let register_result = rt.block_on(local_node.unwrap().register());
        if !local_node_unwrapped.register().await {
            println!("Could not add Node to Node list.");
            std::process::exit(1);
        }
    }

}