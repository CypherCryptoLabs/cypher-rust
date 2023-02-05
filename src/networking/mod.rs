extern crate hyper;
extern crate serde;
extern crate serde_json;

mod route_handler;
mod client;
pub mod node;

use hyper::{service::{ make_service_fn, service_fn}, Server};
use std::{convert::Infallible, io::{Error, ErrorKind}};
use tokio;

#[tokio::main]
pub async fn start_http_server() {

    // We'll bind to 127.0.0.1:3000
    let addr = "0.0.0.0:1234".parse().unwrap();

    let make_service = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(route_handler::handler))
    });

    // Then bind and serve...
    let server = Server::bind(&addr).serve(make_service);

    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

pub async fn register_to_network(seed_node: &node::Node, local_node: &node::Node) -> Result<bool, std::io::Error> {
    if seed_node.ip_address == local_node.ip_address {
        println!("Seed node is the same as local node. This node is seeding a new network!");
        return Ok(true);
    }

    let seed_node_info = client::http_get_request_timeout(seed_node.ip_address.to_owned(), "/".to_string()).await;
    let seed_node_info_unwrapped: String;

    match seed_node_info {
        Ok(_) => {seed_node_info_unwrapped = seed_node_info.unwrap()}
        Err(e) => {return Err(e);}
    }

    let seed_node_info_json: Result<node::NodeInfo, serde_json::Error> = serde_json::from_str(&seed_node_info_unwrapped);
    let seed_node_version: String;

    match seed_node_info_json {
        Ok(_) => {
            let data = seed_node_info_json.unwrap();
            seed_node_version = data.node_version;
        }
        Err(_) => {
            return Err(Error::new(ErrorKind::Other, "Could not convert NodeInfo to json"));
        }
    }

    let node_json = serde_json::to_string(local_node);
    let node_json_string: String;

    match node_json {
        Ok(_) => {node_json_string = node_json.unwrap()}
        Err(_) => {return Err(Error::new(ErrorKind::InvalidData, "Could not stringify Node"));}
    }

    let registration_status = client::http_post_request_timeout(
        seed_node.ip_address.to_owned(), 
        "/v".to_string() + &seed_node_version + "/network/node",
        node_json_string
    ).await;
    let registration_status_unwrapped: String;

    match registration_status {
        Ok(_) => {
            registration_status_unwrapped = registration_status.unwrap();
        }
        Err(e) => {
            println!("{:#?}", e);
            return Err(e);
        }
    }

    let registration_status_json: Result<route_handler::response::PostNode, serde_json::Error> = serde_json::from_str(&registration_status_unwrapped);

    match registration_status_json {
        Ok(_) => {
            return Ok(registration_status_json.unwrap().status);
        }

        Err(_) => {
            return Err(Error::new(ErrorKind::Other, "Could not convert to to PostNode"));
            
        }
    }
}

pub async fn sync_node_list(node: &node::Node) -> bool {
    let seed_node_network = client::http_get_request_timeout(node.ip_address.clone(), "/v".to_string() + &node.version + "/network").await;
    let seed_node_network_unwrapped: String;

    match seed_node_network {
        Ok(_) => {
            seed_node_network_unwrapped = seed_node_network.unwrap();
        }
        Err(e) => {
            println!("Something went wrong with the HTTP request: {:#?}", e);
            return false;
        },
    }

    let seed_node_network_json: Result<route_handler::response::GetNodes, serde_json::Error> = serde_json::from_str(&seed_node_network_unwrapped);
    let seed_node_network_json_unwrapped: route_handler::response::GetNodes;

    match seed_node_network_json {
        Ok(_) => {
            seed_node_network_json_unwrapped = seed_node_network_json.unwrap();
        },
        Err(e) => {
            println!("Something went wrong with the JSON parsing: {:#?}\n{:#?}", e, seed_node_network_unwrapped);
            return false;
        }
    }

    unsafe {
        node::NODE_LIST = seed_node_network_json_unwrapped.nodes;
    }

    return true;
}