extern crate hyper;
extern crate hyper_routing;
extern crate serde;
extern crate serde_json;

mod route_handler;
mod client;
pub mod node;

use hyper::{service::{ make_service_fn, service_fn}, Server};
use std::{convert::Infallible, io::{Error, ErrorKind}};
use tokio;

use self::node::NodeInfo;

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
    let seed_node_registration_timestamp: u64;

    match seed_node_info_json {
        Ok(_) => {
            let data = seed_node_info_json.unwrap();
            seed_node_version = data.node_version;
        }
        Err(_) => {
            return Err(Error::new(ErrorKind::Other, "Could not convert NodeInfo to json"));
        }
    }

    let registration_status = client::http_post_request_timeout(seed_node.ip_address.to_owned(), "/".to_string() + &seed_node_version + "/node", "a".to_string()).await;

    match registration_status {
        Ok(_) => {
            let data = registration_status.unwrap();
            println!("{:#?}", data);

            return Ok(true)
        }
        Err(e) => {
            println!("{:#?}", e);
            return Err(e);
        }
    }

    return Ok(false);
}