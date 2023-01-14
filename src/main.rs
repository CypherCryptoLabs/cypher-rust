extern crate serde;
extern crate serde_json;
extern crate hyper;
extern crate hyper_router;

use hyper::header::{CONTENT_LENGTH, CONTENT_TYPE};
use hyper::{Request, Response, Body, Method};
use hyper::server::Server;
use hyper::rt::Future;
use hyper_router::{Route, RouterBuilder, RouterService};

static _NODE_NAME: &str = env!("CARGO_PKG_NAME");
static _NODE_VERSION: &str = env!("CARGO_PKG_VERSION");

fn request_handler(_: Request<Body>) -> Response<Body> {
    let body = serde_json::to_string(&serde_json::json!({
        "node_name": _NODE_NAME,
        "node_version": _NODE_VERSION
    })).unwrap();

    Response::builder()
        .header(CONTENT_LENGTH, body.len() as u64)
        .header(CONTENT_TYPE, "text/plain")
        .body(Body::from(body))
        .expect("Failed to construct the response")
}

fn router_service() -> Result<RouterService, std::io::Error> {
    let router = RouterBuilder::new()
        .add(Route::get("/version").using(request_handler))
        .add(Route::get(&("/v".to_owned() + _NODE_VERSION)).using(request_handler))
        .build();

    Ok(RouterService::new(router))
}

fn main() {
    let addr = "0.0.0.0:1234".parse().unwrap();
    let server = Server::bind(&addr)
        .serve(router_service)
        .map_err(|e| eprintln!("server error: {}", e));

    hyper::rt::run(server)
}