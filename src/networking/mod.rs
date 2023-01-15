extern crate hyper;
extern crate hyper_router;
extern crate serde;
extern crate serde_json;

mod route_handler;

use hyper::server::Server;
use hyper::rt::Future;

static _NODE_VERSION: &str = env!("CARGO_PKG_VERSION");
static _NODE_NAME: &str = env!("CARGO_PKG_NAME");

pub fn start_http_server() {
    let addr = "0.0.0.0:1234".parse().unwrap();
    let server = Server::bind(&addr)
        .serve(route_handler::router_service)
        .map_err(|e| eprintln!("server error: {}", e));

    hyper::rt::run(server)
}