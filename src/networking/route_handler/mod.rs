extern crate hyper;
extern crate hyper_routing;
extern crate serde;
extern crate serde_json;

use std::convert::Infallible;
use std::pin::Pin;
use futures_util::{ Future};
use hyper::header::{CONTENT_LENGTH, CONTENT_TYPE};
use hyper::{Request, Response, Body, StatusCode, body};
use serde_json::json;

impl Copy for Request<Body> {}
trait Copy {}

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
        Route {path: &"/version", method: &"GET", handler: get_version},
        Route {path: &get_nodes_path, method: &"GET", handler: get_nodes},
        Route {path: &post_nodes_path, method: &"POST", handler: post_node},
    ];

    for route in api_routes.iter() {
        if request_path == route.path &&
            request_method == route.method {
                //let request_static= Box::leak(Box::new(req));
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

fn get_version(_req: Request<Body>) -> Pin<Box<dyn Future<Output = Response<Body>> + Send>> {
    Box::pin(async {
        let body: String = serde_json::to_string(&serde_json::json!({
            "node_name": _NODE_NAME,
            "node_version": _NODE_VERSION
        })).unwrap();

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
            let body: String = serde_json::to_string(&json!(&super::super::consensus::NODE_LIST)).unwrap();

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
        // let req_clone = req.to_owned().clone();
        // let request_body_bytes = req_clone.into_body().filter_map(|b| async {b.ok()}).collect::<Vec<hyper::body::Bytes>>().await;
        // let request_body_bytes_vec: Vec<u8> = request_body_bytes.into_iter().flat_map(|b| b.to_vec()).collect();
        // let data = String::from_utf8(request_body_bytes_vec);

        let body = req.into_body();
        let bytes = body::to_bytes(body).await.unwrap();
        let data = String::from_utf8((&*bytes).to_vec());

        let mut response: Response<Body>;

        match data {
            Ok(_) => {
                println!("{:#?}", data);
                let body = data.unwrap();
                response = Response::builder()
                    .header(CONTENT_LENGTH, body.len() as u64)
                    .header(CONTENT_TYPE, "text/plain")
                    .body(Body::from(body))
                    .expect("Failed to construct the response")
            }
            Err(_) => {
                let body = "404 - Route not supported by Node";
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