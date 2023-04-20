extern crate hyper;
extern crate serde;
extern crate serde_json;

pub mod response;
pub mod request;

use futures_util::{ Future};
use hyper::header::{CONTENT_LENGTH, CONTENT_TYPE};
use hyper::{Request, Response, Body, StatusCode, body};
use std::convert::Infallible;
use std::pin::Pin;
use std::time::{SystemTime, UNIX_EPOCH};
use hex;
use crate::blockchain::Block;
use crate::networking::node::{LOCAL_BLOCKCHAIN_ADDRESS, NODE_LIST};

use super::super::transaction_queue;

use self::request::Node;
use self::request::Tx;

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
    timestamp: u64,
    public_key: String
}

impl<T: serde::Serialize+Clone> MetaDataPreSignature<T> {
    
}

impl<T: serde::Serialize+Clone+std::fmt::Debug+ for<'a> serde::Deserialize<'a>> MetaData<T> {

    pub unsafe fn new(payload:T) -> MetaData<T> {

        let public_key_string = hex::encode(&super::super::crypto::PUBLIC_KEY.unwrap().serialize_uncompressed().to_vec());
        let data = MetaDataPreSignature {
            payload: payload,
            public_key: public_key_string,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)
                    .expect("Time went backwards").as_millis() as u64,
        };

        let meta_data_string = serde_json::to_string(&data).unwrap();
        let signature = super::super::crypto::sign_string(&meta_data_string);

        return MetaData { payload: data.payload, signature, public_key: data.public_key, timestamp: data.timestamp };

    }

    pub fn to_string(self) -> String {
        return serde_json::to_string(&self).unwrap();
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

    pub async unsafe fn broadcast(self, endpoint: String) {
        let mut notified_random_peers: Vec<Node> = vec![];
        let mut random_peers_successfully_notified = 0;
        let n_random_peers: i32 = if super::node::NODE_LIST.len() > 8 {
            8 as i32
        } else {
            super::node::NODE_LIST.len() as i32
        };
        let self_str = serde_json::to_string(&self).unwrap();
        let mut rng = <::rand::rngs::StdRng as rand::SeedableRng>::from_seed(rand::Rng::gen(&mut rand::rngs::OsRng));

        println_debug!("{}", self_str);
        println_debug!("{}", n_random_peers);

        let local_node = NODE_LIST.iter().find(|s| s.blockchain_address.eq(&LOCAL_BLOCKCHAIN_ADDRESS.to_string())).unwrap();
        notified_random_peers.push(local_node.to_owned());

        while (notified_random_peers.len() as i32) < n_random_peers && random_peers_successfully_notified < n_random_peers {
            let random_node_index = rand::Rng::gen_range(&mut rng, 0..super::node::NODE_LIST.len());
            let random_node = super::node::NODE_LIST[random_node_index].to_owned();
    
            if notified_random_peers.contains(&random_node) {
                continue;
            }
            
            println_debug!("sending packet to {}", random_node.ip_address);
            notified_random_peers.push(random_node.to_owned());
            let http_post_result = super::client::http_post_request_timeout(
                random_node.ip_address.clone(), 
                "/v".to_string() + &random_node.version + &endpoint,
                self_str.clone()
            ).await;
            
            let broadcast_status = match http_post_result {
                Ok(result) => {
                    result
                },
                Err(e) => {
                    println_debug!("{:#?}", e);
                    continue;
                }
            };
    
            let broadcast_status_json: MetaData<response::Broadcast> = match serde_json::from_str(&broadcast_status) {
                Ok(result) => {
                    result
                },
                Err(e) => {
                    println_debug!("[{}] {:#?}: {}", random_node.ip_address, e, broadcast_status);
                    continue;
                }
            };

            if broadcast_status_json.payload.status {
                random_peers_successfully_notified += 1;
            }
    
        }

        println_debug!("{:#?}", notified_random_peers);

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
    let post_tx_path = "/v".to_owned() + _NODE_VERSION + "/blockchain/tx";
    let post_block_path = "/v".to_owned() + _NODE_VERSION + "/blockchain/propose";

    let api_routes: Vec<Route> = vec![
        Route {path: &"/", method: &"GET", handler: get_info},
        Route {path: &get_nodes_path, method: &"GET", handler: get_nodes},
        Route {path: &post_nodes_path, method: &"POST", handler: post_node},
        Route {path: &post_tx_path, method: &"POST", handler: post_tx},
        Route {path: &post_block_path, method: &"POST", handler: post_block_propose},
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
            }).to_string();
        

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
            let body: String = MetaData::new(response::GetNodes{nodes: super::node::NODE_LIST.clone()}).to_string();

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
        let bytes = match body::to_bytes(body).await {
            Ok(result) => {
                result
            },
            Err(e) => {
                println_debug!("{}", e);
                hyper::body::Bytes::from("")
            }
        };
        let data = String::from_utf8((&*bytes).to_vec());

        let body = "400 - Malformed Request";
        let mut response = Response::builder()
            .header(CONTENT_LENGTH, body.len() as u64)
            .header(CONTENT_TYPE, "text/plain")
            .body(Body::from(body))
            .expect("Failed to construct the response");

        let request_body_str: String;
        match data {
            Ok(_) => {
                request_body_str = data.unwrap();
            }
            Err(_) => {
                return response;
            }
        }

        let request_body_json:Result<MetaData<Node>, serde_json::Error> = serde_json::from_str(request_body_str.as_str());
        let mut new_node: Node;
        match request_body_json {
            Ok(_) => {
                if request_body_json.as_ref().unwrap().verify() {
                    new_node = request_body_json.unwrap().payload;
                } else {
                    return response;
                }
            },
            Err(_) => {
                return response;
            },
        }

        unsafe {
            let register_success = new_node.register().await;

            let registration_body= MetaData::new(response::Broadcast{status: register_success}).to_string();

            response = Response::builder()
                .header(CONTENT_LENGTH, registration_body.len() as u64)
                .header(CONTENT_TYPE, "text/plain")
                .body(Body::from(registration_body))
                .expect("Failed to construct the response");
        }

        return response;
    })
}

fn post_tx(req: Request<Body>) -> Pin<Box<dyn Future<Output = Response<Body>> + Send>> {
    Box::pin(async move {
        let body = req.into_body();
        let bytes = body::to_bytes(body).await.unwrap();
        let data = String::from_utf8((&*bytes).to_vec());

        let body = "400 - Malformed Request";
        let mut response = Response::builder()
            .header(CONTENT_LENGTH, body.len() as u64)
            .header(CONTENT_TYPE, "text/plain")
            .body(Body::from(body))
            .expect("Failed to construct the response");

        let request_body_str: String;
        match data {
            Ok(_) => {
                request_body_str = data.unwrap();
            }
            Err(e) => {
                println_debug!("{:#?}", e);
                return response;
            }
        }

        let request_body_json:Result<MetaData<Tx>, serde_json::Error> = serde_json::from_str(request_body_str.as_str());
        match request_body_json {
            Ok(request_body_json_unwrapped) => {
                if request_body_json_unwrapped.verify() {

                    println_debug!("{}", request_body_str);

                    let tx_is_not_known = transaction_queue::get(&request_body_json_unwrapped.payload.signature).is_none();

                    if tx_is_not_known {
                        transaction_queue::insert(&request_body_json_unwrapped.payload);
                        unsafe {tokio::spawn({

                            let request_body_json_unwrapped_clone = request_body_json_unwrapped.clone();
                            request_body_json_unwrapped_clone.broadcast("/blockchain/tx".to_string())
                        })};
                    }

                    let new_body: String = unsafe { MetaData::new(response::Broadcast{status: tx_is_not_known}).to_string() };
                    response = Response::builder()
                        .header(CONTENT_LENGTH, new_body.len() as u64)
                        .header(CONTENT_TYPE, "text/plain")
                        .body(Body::from(new_body))
                        .expect("Failed to construct the response");
                } else {
                    return response;
                }
            },
            Err(e) => {
                println_debug!("{:#?}\n{}", e, request_body_str);
                return response;
            },
        }

        return response;
    })
}

fn post_block_propose(req: Request<Body>) -> Pin<Box<dyn Future<Output = Response<Body>> + Send>> {
    Box::pin(async move {
        let body = req.into_body();
        let bytes = body::to_bytes(body).await.unwrap();
        let data = String::from_utf8((&*bytes).to_vec());

        let body = "400 - Malformed Request";
        let mut response = Response::builder()
            .header(CONTENT_LENGTH, body.len() as u64)
            .header(CONTENT_TYPE, "text/plain")
            .body(Body::from(body))
            .expect("Failed to construct the response");

        if unsafe { 
            super::CURRENT_VALIDATORS.iter().all(|n| n.blockchain_address != super::node::LOCAL_BLOCKCHAIN_ADDRESS.to_string()) 
        } {
            return response;
        }

        let request_body_str: String = match data {
            Ok(data) => {
                data
            }
            Err(e) => {
                println_debug!("{:#?}", e);
                return response;
            }
        };

        let request_body_json:MetaData<Block> = match serde_json::from_str(request_body_str.as_str()) {
            Ok(data) => {
                data
            },
            Err(e) => {
                println_debug!("{:#?}", e);
                return response;
            },
        };

        let proposed_block = request_body_json.payload;

        if unsafe { super::super::queue_worker::CURRENT_PROPOSED_BLOCK.forger_signature == proposed_block.forger_signature} {
            let new_body: String = unsafe { MetaData::new(response::Broadcast{status: false}).to_string() };
            response = Response::builder()
                .header(CONTENT_LENGTH, new_body.len() as u64)
                .header(CONTENT_TYPE, "text/plain")
                .body(Body::from(new_body))
                .expect("Failed to construct the response");

            return response;
        }
        
        unsafe { super::super::queue_worker::CURRENT_PROPOSED_BLOCK = proposed_block.clone() };

        tokio::spawn(async move {
            proposed_block.broadcast_to_validators(unsafe { super::CURRENT_VALIDATORS.clone() }).await;
        });

        let new_body: String = unsafe { MetaData::new(response::Broadcast{status: true}).to_string() };
        response = Response::builder()
            .header(CONTENT_LENGTH, new_body.len() as u64)
            .header(CONTENT_TYPE, "text/plain")
            .body(Body::from(new_body))
            .expect("Failed to construct the response");

        return response;
    })
}