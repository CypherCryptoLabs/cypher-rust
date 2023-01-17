extern crate hyper;
extern crate hyper_routing;
extern crate serde;
extern crate serde_json;

use std::convert::Infallible;

use futures_util::{TryStreamExt};
use hyper::header::{CONTENT_LENGTH, CONTENT_TYPE};
use hyper::{Request, Response, Body, StatusCode};
use serde_json::json;

static _NODE_VERSION: &str = env!("CARGO_PKG_VERSION");
static _NODE_NAME: &str = env!("CARGO_PKG_NAME");
static mut API_ENDPOINTS: Vec<(&str, Box<dyn Fn(Request<Body>) -> Response<Body>>, &str)> = vec![
    (&("/version"), unsafe {std::mem::transmute(&get_version)}, "GET"),
    (&("/v".to_owned() + _NODE_VERSION + "/network"), unsafe {std::mem::transmute(&get_nodes)}, "GET"),
    (&("/v".to_owned() + _NODE_VERSION + "/network/node"), unsafe {std::mem::transmute(&post_node)}, "POST"),
];

// pub fn router_service() -> RouterService {
//     let router = RouterBuilder::new()
//         .add(Route::get("/version").using(get_version))
//         .add(Route::get(&("/v".to_owned() + _NODE_VERSION + "/network")).using(get_nodes))
//         .add(Route::post(&("/v".to_owned() + _NODE_VERSION + "/network/node")).using(post_node))
//         .add(Route::get("*").using(|_req| {
//             let custom_404 = "404 - Route not supported by this Node.";
//             let mut response = Response::new(Body::from(custom_404));
//             *response.status_mut() = StatusCode::NOT_FOUND;
//             response
//         }))
//         .build();

//     RouterService::new(router)
// }

pub async fn handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let request_path: &str = req.uri().path();
    let request_method = req.method().to_string();
    let request_method = request_method.as_str();
    let mut response = Response::new(Body::from("404 - Route not supported by this Node."));
    *response.status_mut() = StatusCode::NOT_FOUND;

    unsafe {
        let api_endpoints_ref = &API_ENDPOINTS;
        for (request_path, request_handler, request_method) in api_endpoints_ref.into_iter() {

            if request_path == request_path && request_method == request_method {
                response = request_handler(req);
                break;
            }
        }
    }

    Ok(response)

}

async fn get_version(_: Request<Body>) -> Response<Body> {
    
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
}

async fn get_nodes(_: Request<Body>) -> Response<Body> {
    
    unsafe {
        let body: String = serde_json::to_string(&json!(&super::super::consensus::NODE_LIST)).unwrap();

        let response = Response::builder()
            .header(CONTENT_LENGTH, body.len() as u64)
            .header(CONTENT_TYPE, "text/plain")
            .body(Body::from(body))
            .expect("Failed to construct the response");

        response
    }
}


async fn post_node(req: Request<Body>) -> Response<Body> {
    
    let request_body_bytes = req.into_body().try_next().await.unwrap().unwrap();
    let data = String::from_utf8(request_body_bytes.to_vec());
    
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
            let body = "404 - Route not supported by this Node.".to_string();
            response = Response::builder()
                    .header(CONTENT_LENGTH, body.len() as u64)
                    .header(CONTENT_TYPE, "text/plain")
                    .body(Body::from(body))
                    .expect("Failed to construct the response");

            *response.status_mut() = StatusCode::BAD_REQUEST;
        },
    }

    response


}