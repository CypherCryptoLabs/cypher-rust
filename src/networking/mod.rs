extern crate hyper;
extern crate hyper_router;
extern crate serde;
extern crate serde_json;

mod route_handler;

use hyper::{Response, Body, StatusCode};
use hyper::server::Server;
use hyper::rt::Future;
use hyper_router::{Route, RouterBuilder, RouterService};

static _NODE_VERSION: &str = env!("CARGO_PKG_VERSION");
static _NODE_NAME: &str = env!("CARGO_PKG_NAME");

fn router_service() -> Result<RouterService, std::io::Error> {
    let router = RouterBuilder::new()
        .add(Route::get("/version").using(route_handler::get_version))
        .add(Route::get(&("/v".to_owned() + _NODE_VERSION + "/network")).using(route_handler::get_nodes))
        .add(Route::get("*").using(|_req| {
            let custom_404 = "404 - Route not supported by this Node.";
            let mut response = Response::new(Body::from(custom_404));
            *response.status_mut() = StatusCode::NOT_FOUND;
            response
        }))
        .build();

        Ok(RouterService::new(router))
}

pub fn start_http_server() {
    let addr = "0.0.0.0:1234".parse().unwrap();
    let server = Server::bind(&addr)
        .serve(router_service)
        .map_err(|e| eprintln!("server error: {}", e));

    hyper::rt::run(server)
}