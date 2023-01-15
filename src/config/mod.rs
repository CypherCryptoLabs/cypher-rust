extern crate regex;

use lazy_static::lazy_static;
use regex::Regex;
use std::env;

lazy_static! {
    pub static ref IP_ADDRESS: String = env::var("CYPHER_EXTERNAL_IP").unwrap_or_default();
}

pub fn load() {
    match IP_ADDRESS.as_str() {
        "" => {
            println!("'CYPHER_EXTERNAL_IP' not set!");
            std::process::exit(1);
        },
        &_ => {
            if !Regex::new(r"^(([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])\.){3}([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])$")
                .unwrap()
                .is_match(&IP_ADDRESS)
            {
                std::process::exit(1);
            }
        }
    }
}