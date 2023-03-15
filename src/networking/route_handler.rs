extern crate hyper;
extern crate serde;
extern crate serde_json;

pub mod response;

use bitcoin::util::address::Payload;
use futures_util::future::ok;
use futures_util::{ Future};
use hyper::header::{CONTENT_LENGTH, CONTENT_TYPE};
use hyper::{Request, Response, Body, StatusCode, body};
use secp256k1::PublicKey;
use std::convert::Infallible;
use std::error::Error;
use std::pin::Pin;
use std::time::{SystemTime, UNIX_EPOCH};
use hex;

use super::node::Node;

struct Route<'a> {
    path: &'a str,
    method: &'a str,
    handler:  fn(Request<Body>) -> Pin<Box<dyn Future<Output = Response<Body>> + Send>>
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct MetaData<T> {
    pub payload: T,
    signature: String,
    public_key: String,
    timestamp: u64
}

#[derive(serde::Serialize, serde::Deserialize)]
struct MetaDataPreSignature<T>{
    payload: T,
    public_key: String,
    timestamp: u64
}

impl<T: serde::Serialize+Clone> MetaData<T> {

    pub unsafe fn new(payload:T) -> String{

        let public_key_string = hex::encode(&super::super::crypto::PUBLIC_KEY.unwrap().serialize_uncompressed().to_vec());
        let data = MetaDataPreSignature {
            payload: payload,
            public_key: public_key_string,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)
                    .expect("Time went backwards").as_millis() as u64,
        };

        let meta_data_string = serde_json::to_string(&data).unwrap();
        let signature = super::super::crypto::sign_string(&meta_data_string);

        return serde_json::to_string(&MetaData { payload: data.payload, signature, public_key: data.public_key, timestamp: data.timestamp }).unwrap();

    }

    pub fn verify(&self) -> bool {
        let result = || -> Result<bool, serde_json::error::Error> {
            let data = MetaDataPreSignature {
                payload: self.payload.clone(),
                public_key: self.public_key.clone(),
                timestamp: self.timestamp
            };

            let message_string = serde_json::to_string(&data)?;
            let signature_ok = super::super::crypto::verify_signature(&self.signature, &message_string, &self.public_key);

            return Ok(signature_ok);
        }();

        match result {
            Ok(_) => {return result.unwrap();},
            Err(_) => {return false;}
        }
    }

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
        unsafe{
            let body = MetaData::new(response::GetInfo{
                node_name: _NODE_NAME.to_string(),
                node_version: _NODE_VERSION.to_string(),
                unix_time: SystemTime::now().duration_since(UNIX_EPOCH)
                .expect("Time went backwards").as_millis() as u64,
                blockchain_address: super::node::LOCAL_BLOCKCHAIN_ADDRESS.to_string()
            });
        

            let response = Response::builder()
                .header(CONTENT_LENGTH, body.len() as u64)
                .header(CONTENT_TYPE, "text/plain")
                .body(Body::from(body))
                .expect("Failed to construct the response");

            response
        }
    })
}

fn get_nodes(_req: Request<Body>) -> Pin<Box<dyn Future<Output = Response<Body>> + Send>> {
    Box::pin(async move {
        unsafe {
            let body: String = MetaData::new(response::GetNodes{nodes: super::node::NODE_LIST.clone()});

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
                let request_body_json:Result<MetaData<super::node::Node>, serde_json::Error> = serde_json::from_str(request_body_str.as_str());

                match request_body_json {
                    Ok(_) => {
                        let mut new_node = request_body_json.unwrap().payload;
                        unsafe { 
                            let register_success = new_node.register().await;

                            let body: String = MetaData::new(response::PostNode{status: register_success});

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