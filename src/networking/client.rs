use std::{io::{Error, ErrorKind}, str::FromStr};

use hyper::{Client, Uri};

pub async fn http_request_timeout(ip_address: String) -> Result<String, Error> {
    let client:Client<hyper::client::HttpConnector> = Client::builder()
        .pool_idle_timeout(std::time::Duration::from_secs(3))
        .http2_only(true)
        .build_http();
    let node_uri = Uri::from_str(&("http://".to_owned() + &ip_address + ":1234/")).unwrap();
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