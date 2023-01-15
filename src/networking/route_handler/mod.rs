extern crate hyper;
extern crate hyper_router;
extern crate serde;
extern crate serde_json;

use hyper::header::{CONTENT_LENGTH, CONTENT_TYPE};
use hyper::{Request, Response, Body};

static _NODE_VERSION: &str = env!("CARGO_PKG_VERSION");
static _NODE_NAME: &str = env!("CARGO_PKG_NAME");

pub fn get_version(_: Request<Body>) -> Response<Body> {
    
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

pub fn get_nodes(_: Request<Body>) -> Response<Body> {
    
    let body: String = serde_json::to_string(&serde_json::json!([])).unwrap();

    Response::builder()
        .header(CONTENT_LENGTH, body.len() as u64)
        .header(CONTENT_TYPE, "text/plain")
        .body(Body::from(body))
        .expect("Failed to construct the response")
}