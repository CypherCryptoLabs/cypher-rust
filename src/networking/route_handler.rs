extern crate hyper;
extern crate hyper_routing;
extern crate serde;
extern crate serde_json;

pub mod response;

use futures_util::{ Future};
use hyper::header::{CONTENT_LENGTH, CONTENT_TYPE};
use hyper::{Request, Response, Body, StatusCode, body};
use std::convert::Infallible;
use std::pin::Pin;
use std::time::{SystemTime, UNIX_EPOCH};

struct Route<'a> {
    path: &'a str,
    method: &'a str,
    handler:  fn(Request<Body>) -> Pin<Box<dyn Future<Output = Response<Body>> + Send>>
}

static _NODE_VERSION: &str = env!("CARGO_PKG_VERSION");
static _NODE_NAME: &str = env!("CARGO_PKG_NAME");

pub async fn handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {

    //let request_borrowed: &Request<Body> = &req;
    let request_path: &str = req.uri().path();
    let request_method: String = req.method().to_string();
    let request_method: &str = request_method.as_str();
    let mut response: Option<Response<Body>> = None;

    let get_nodes_path = "/v".to_owned() + _NODE_VERSION + "/network";
    let post_nodes_path = "/v".to_owned() + _NODE_VERSION + "/network/node";

    let api_routes: Vec<Route> = vec![
        Route {path: &"/", method: &"GET", handler: get_info},
        Route {path: &get_nodes_path, method: &"GET", handler: get_nodes},
        Route {path: &post_nodes_path, method: &"POST", handler: post_node},
    ];

    for route in api_routes.iter() {
        if request_path == route.path &&
            request_method == route.method {
                response = Some((route.handler)(Request::new(req.into_body())).await);
                break;
            }
    }

    if response.is_none() {
        let body: String = "404 - Route not supported by this Node.".to_string();
        let mut error_response: Response<Body> = Response::builder()
                .header(CONTENT_LENGTH, body.len() as u64)
                .header(CONTENT_TYPE, "text/plain")
                .body(Body::from(body))
                .expect("Failed to construct the response");

            *error_response.status_mut() = StatusCode::NOT_FOUND;

        response = Some(error_response)
    }

    Ok(response.unwrap())

}

fn get_info(_req: Request<Body>) -> Pin<Box<dyn Future<Output = Response<Body>> + Send>> {
    Box::pin(async {
        let body: String = serde_json::to_string(
            &response::GetInfo {
                node_name: _NODE_NAME.to_string(),
                node_version: _NODE_VERSION.to_string(),
                unix_time: SystemTime::now().duration_since(UNIX_EPOCH)
                .expect("Time went backwards").as_millis() as u64,
                blockchain_address: super::node::LOCAL_BLOCKCHAIN_ADDRESS.to_string()
            }
        ).unwrap();

        let response = Response::builder()
            .header(CONTENT_LENGTH, body.len() as u64)
            .header(CONTENT_TYPE, "text/plain")
            .body(Body::from(body))
            .expect("Failed to construct the response");

        response
    })
}

fn get_nodes(_req: Request<Body>) -> Pin<Box<dyn Future<Output = Response<Body>> + Send>> {
    Box::pin(async move {
        unsafe {
            let body: String = serde_json::to_string(&response::GetNodes{nodes: super::node::NODE_LIST.clone()}).unwrap();

            let response = Response::builder()
                .header(CONTENT_LENGTH, body.len() as u64)
                .header(CONTENT_TYPE, "text/plain")
                .body(Body::from(body))
                .expect("Failed to construct the response");

            response
        }
    })
}

fn post_node(req: Request<Body>) -> Pin<Box<dyn Future<Output = Response<Body>> + Send>> {
    Box::pin(async move {
        let body = req.into_body();
        let bytes = body::to_bytes(body).await.unwrap();
        let data = String::from_utf8((&*bytes).to_vec());

        let mut response: Response<Body>;

        match data {
            Ok(_) => {
                let request_body_str = data.unwrap();
                let request_body_json:Result<super::node::Node, serde_json::Error> = serde_json::from_str(request_body_str.as_str());

                match request_body_json {
                    Ok(_) => {
                        let new_node = request_body_json.unwrap();
                        unsafe { 
                            let register_success = new_node.register();

                            let body: String = serde_json::to_string(&response::PostNode{status: register_success.await,}).unwrap();

                            response = Response::builder()
                                .header(CONTENT_LENGTH, body.len() as u64)
                                .header(CONTENT_TYPE, "text/plain")
                                .body(Body::from(body))
                                .expect("Failed to construct the response")
                        }
                    }

                    Err(_) => {
                        let body = "400 - Malformed Request";
                        response = Response::builder()
                                .header(CONTENT_LENGTH, body.len() as u64)
                                .header(CONTENT_TYPE, "text/plain")
                                .body(Body::from(body))
                                .expect("Failed to construct the response");

                        *response.status_mut() = StatusCode::BAD_REQUEST;
                    }
                }
            }
            Err(_) => {
                let body = "400 - Malformed Request";
                response = Response::builder()
                        .header(CONTENT_LENGTH, body.len() as u64)
                        .header(CONTENT_TYPE, "text/plain")
                        .body(Body::from(body))
                        .expect("Failed to construct the response");

                *response.status_mut() = StatusCode::BAD_REQUEST;
            },
        }

        response
    })
}