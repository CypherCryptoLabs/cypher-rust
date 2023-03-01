extern crate regex;

use lazy_static::lazy_static;
use regex::Regex;
use std::env;

lazy_static! {
    pub static ref IP_ADDRESS: String = env::var("CYPHER_EXTERNAL_IP").unwrap_or_default();
    pub static ref SEED_IP_ADDRESS: String = env::var("CYPHER_SEED_IP").unwrap_or_default();
    pub static ref SEED_WALLET_ADDRESS: String = env::var("CYPHER_SEED_WALLET_ADDRESS").unwrap_or_default();
    pub static ref SEED_VERSION: String = env::var("CYPHER_SEED_VERSION").unwrap_or_default();
    pub static ref SEED_PHRASE_PATH: String = env::var("CYPHER_SEED_PHRASE_PATH").unwrap_or("/data/seed_phrase.txt".to_string());
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
                println!("'CYPHER_EXTERNAL_IP' is not set correctly!");
                std::process::exit(1);
            }
        }
    }

    match SEED_IP_ADDRESS.as_str() {
        "" => {
            println!("'CYPHER_SEED_IP' not set!");
            std::process::exit(1);
        },
        &_ => {
            if !Regex::new(r"^(([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])\.){3}([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])$")
                .unwrap()
                .is_match(&SEED_IP_ADDRESS)
            {
                println!("'CYPHER_SEED_IP' is not set correctly!");
                std::process::exit(1);
            }
        }
    }

    match SEED_WALLET_ADDRESS.as_str() {
        "" => {
            println!("'CYPHER_SEED_WALLET_ADDRESS' not set!");
            std::process::exit(1);
        },
        &_ => {
            if !Regex::new(r"^0x[a-fA-F0-9]{40}$")
                .unwrap()
                .is_match(&SEED_WALLET_ADDRESS)
            {
                println!("'CYPHER_SEED_WALLET_ADDRESS' is not set correctly!");
                std::process::exit(1);
            }
        }
    }

    match SEED_VERSION.as_str() {
        "" => {
            println!("'CYPHER_SEED_VERSION' not set!");
            std::process::exit(1);
        },
        &_ => {
            if !Regex::new(r"^\d+\.\d+\.\d+$")
                .unwrap()
                .is_match(&SEED_VERSION)
            {
                println!("'CYPHER_SEED_VERSION' is not set correctly!");
                std::process::exit(1);
            }
        }
    }

    // match SEED_PHRASE_PATH.as_str() {
    //     "" => {
    //         println!("'CYPHER_SEED_PHRASE_PATH' not set, using default value!");
    //     },
    //     &_ => {
    //         if !Regex::new(r"^(/[a-zA-Z0-9._-]+)+$")
    //             .unwrap()
    //             .is_match(&SEED_PHRASE_PATH)
    //         {
    //             println!("'CYPHER_SEED_PHRASE_PATH' is not set correctly!");
    //             std::process::exit(1);
    //         }
    //     }
    // }
}