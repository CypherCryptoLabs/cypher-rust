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
        println_debug!("server error: {}", e);
    }
}

pub async fn register_to_network(seed_node: &node::Node, local_node: &node::Node) -> Result<bool, std::io::Error> {
    if seed_node.ip_address == local_node.ip_address {
        println_debug!("Seed node is the same as local node. This node is seeding a new network!");
        return Ok(true);
    }

    let seed_node_info = client::http_get_request_timeout(seed_node.ip_address.to_owned(), "/".to_string()).await;
    let seed_node_info_unwrapped: String;

    match seed_node_info {
        Ok(_) => {seed_node_info_unwrapped = seed_node_info.unwrap()}
        Err(e) => {return Err(e);}
    }

    let seed_node_info_json: Result<route_handler::MetaData<node::NodeInfo>, serde_json::Error> = serde_json::from_str(&seed_node_info_unwrapped);
    let seed_node_version: String;

    match seed_node_info_json {
        Ok(_) => {
            let data = seed_node_info_json.unwrap().payload;
            seed_node_version = data.node_version;
        }
        Err(e) => {
            println_debug!("{:#?}", e);
            return Err(Error::new(ErrorKind::Other, e));
        }
    }

    let node_json_string: String = unsafe{route_handler::MetaData::new(local_node)};

    println_debug!("{:#?}", node_json_string);

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
            println_debug!("{:#?}", e);
            return Err(e);
        }
    }

    let registration_status_json: Result<route_handler::MetaData<route_handler::response::PostNode>, serde_json::Error> = serde_json::from_str(&registration_status_unwrapped);

    match registration_status_json {
        Ok(_) => {
            return Ok(registration_status_json.unwrap().payload.status);
        }

        Err(e) => {
            return Err(Error::new(ErrorKind::Other, e));
            
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
            println_debug!("Something went wrong with the HTTP request: {:#?}", e);
            return false;
        },
    }

    let seed_node_network_json: Result<route_handler::MetaData<route_handler::response::GetNodes>, serde_json::Error> = serde_json::from_str(&seed_node_network_unwrapped);
    let seed_node_network_json_unwrapped: route_handler::response::GetNodes;

    match seed_node_network_json {
        Ok(_) => {
            seed_node_network_json_unwrapped = seed_node_network_json.unwrap().payload;
        },
        Err(e) => {
            println_debug!("Something went wrong with the JSON parsing: {:#?}\n{:#?}", e, seed_node_network_unwrapped);
            return false;
        }
    }

    unsafe {
        node::NODE_LIST = seed_node_network_json_unwrapped.nodes;
    }

    return true;
}