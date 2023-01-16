extern crate hyper;
extern crate hyper_routing;
extern crate serde;
extern crate serde_json;

mod route_handler;

use std::task::{Context, Poll};
use futures_util::future;
use hyper::service::Service;
use hyper_routing::{RouterService};
use tokio;

const _NODE_VERSION: &str = env!("CARGO_PKG_VERSION");
const _NODE_NAME: &str = env!("CARGO_PKG_NAME");

pub struct MakeSvc;

impl<T> Service<T> for MakeSvc {
  type Response = RouterService;
  type Error = std::io::Error;
  type Future = future::Ready<Result<Self::Response, Self::Error>>;

  fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
    Ok(()).into()
  }

  fn call(&mut self, _: T) -> Self::Future {
    future::ok(route_handler::router_service())
  }
}


#[tokio::main]
pub async fn start_http_server() {

    // We'll bind to 127.0.0.1:3000
    let addr = "0.0.0.0:1234".parse().unwrap();

    let server = hyper::Server::bind(&addr).serve(MakeSvc);

    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}