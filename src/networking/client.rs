use std::{io::{Error, ErrorKind}, str::FromStr};

use hyper::{Client, Uri, Request, Body};

pub async fn http_get_request_timeout(ip_address: String, endpoint: String) -> Result<String, Error> {
    let client:Client<hyper::client::HttpConnector> = Client::builder()
        .pool_idle_timeout(std::time::Duration::from_secs(3))
        .http2_only(true)
        .build_http();
    let node_uri = Uri::from_str(&("http://".to_owned() + &ip_address + ":1234" + &endpoint)).unwrap();
    let response = client.get(node_uri);

    let response_timeout = tokio::time::timeout(std::time::Duration::from_millis(3000), response).await;
    let response_timeout_unwrapped: Result<hyper::Response<hyper::Body>, hyper::Error>;

    match response_timeout {
        Ok(_) => {
            response_timeout_unwrapped = response_timeout.unwrap()
        }
        Err(_) => {
            let err = response_timeout.err();
            println!("Timeout occured when checking a Nodes reachability: {:#?}", err);
            return Err(Error::new(ErrorKind::TimedOut, "Timeout occured when checking a Nodes reachability"));
        }
    }

    let body_string : String;

    match response_timeout_unwrapped {
        Ok(_) => {
            body_string = String::from_utf8((hyper::body::to_bytes(response_timeout_unwrapped.unwrap()).await.unwrap()).to_vec()).unwrap();
        }
        Err(_) => {
            let err = response_timeout_unwrapped.err();
            println!("{:#?}", err);
            return Err(Error::new(ErrorKind::InvalidData, "Data could not be stringified."));
        }
    }

    return Ok(body_string);

    
}

pub async fn http_post_request_timeout(ip_address: String, endpoint: String, body: String) -> Result<String, Error> {

    let node_uri = Uri::from_str(&("http://".to_owned() + &ip_address + ":1234" + &endpoint)).unwrap();
    let client:Client<hyper::client::HttpConnector> = Client::builder()    
        .pool_idle_timeout(std::time::Duration::from_secs(3))
        .http2_only(true)
        .build_http();

    let request = Request::builder()
        .method(hyper::Method::POST)
        .uri(node_uri)
        .body(Body::from(body))
        .unwrap();

    let response = client.request(request);

    let response_timeout = tokio::time::timeout(std::time::Duration::from_millis(3000), response).await;
    let response_timeout_unwrapped: Result<hyper::Response<hyper::Body>, hyper::Error>;

    match response_timeout {
        Ok(_) => {
            response_timeout_unwrapped = response_timeout.unwrap()
        }
        Err(_) => {
            let err = response_timeout.err();
            println!("Timeout occured when checking a Nodes reachability: {:#?}", err);
            return Err(Error::new(ErrorKind::TimedOut, "Timeout occured when checking a Nodes reachability"));
        }
    }

    let body_string : String;

    match response_timeout_unwrapped {
        Ok(_) => {
            body_string = String::from_utf8((hyper::body::to_bytes(response_timeout_unwrapped.unwrap()).await.unwrap()).to_vec()).unwrap();
        }
        Err(_) => {
            let err = response_timeout_unwrapped.err();
            println!("{:#?}", err);
            return Err(Error::new(ErrorKind::InvalidData, "Data could not be stringified."));
        }
    }

    return Ok(body_string);
}