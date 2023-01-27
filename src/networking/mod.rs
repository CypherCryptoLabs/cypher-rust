extern crate hyper;
extern crate hyper_routing;
extern crate serde;
extern crate serde_json;

mod route_handler;
mod client;
pub mod node;

use hyper::{service::{ make_service_fn, service_fn}, Server};
use std::{convert::Infallible};
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