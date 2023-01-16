extern crate hyper;
extern crate hyper_routing;
extern crate serde;
extern crate serde_json;

use hyper::header::{CONTENT_LENGTH, CONTENT_TYPE};
use hyper::{Request, Response, Body, StatusCode};
use hyper_routing::{Route, RouterBuilder, RouterService};
use serde_json::json;

static _NODE_VERSION: &str = env!("CARGO_PKG_VERSION");
static _NODE_NAME: &str = env!("CARGO_PKG_NAME");

pub fn router_service() -> RouterService {
    let router = RouterBuilder::new()
        .add(Route::get("/version").using(get_version))
        .add(Route::get(&("/v".to_owned() + _NODE_VERSION + "/network")).using(get_nodes))
        .add(Route::post(&("/v".to_owned() + _NODE_VERSION + "/network/node")).using(post_node))
        .add(Route::get("*").using(|_req| {
            let custom_404 = "404 - Route not supported by this Node.";
            let mut response = Response::new(Body::from(custom_404));
            *response.status_mut() = StatusCode::NOT_FOUND;
            response
        }))
        .build();

    RouterService::new(router)
}

fn get_version(_: Request<Body>) -> Response<Body> {
    
    let body: String = serde_json::to_string(&serde_json::json!({
        "node_name": _NODE_NAME,
        "node_version": _NODE_VERSION
    })).unwrap();

    Response::builder()
        .header(CONTENT_LENGTH, body.len() as u64)
        .header(CONTENT_TYPE, "text/plain")
        .body(Body::from(body))
        .expect("Failed to construct the response")
}

fn get_nodes(_: Request<Body>) -> Response<Body> {
    
    unsafe {
        let body: String = serde_json::to_string(&json!(&super::super::consensus::NODE_LIST)).unwrap();

        Response::builder()
            .header(CONTENT_LENGTH, body.len() as u64)
            .header(CONTENT_TYPE, "text/plain")
            .body(Body::from(body))
            .expect("Failed to construct the response")
    }
}

fn post_node(_: Request<Body>) -> Response<Body> {

    let body: String = "".to_string();

    Response::builder()
            .header(CONTENT_LENGTH, body.len() as u64)
            .header(CONTENT_TYPE, "text/plain")
            .body(Body::from(body))
            .expect("Failed to construct the response")
}