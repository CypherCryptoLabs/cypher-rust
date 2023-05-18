use std::process::exit;

use mysql::*;
use mysql::prelude::*;

use crate::blockchain::Block;

static mut CONN: once_cell::sync::Lazy<PooledConn> = once_cell::sync::Lazy::new(|| {
    let url = "mysql://root@127.0.0.1/cypher";

    let pool = match Pool::new(url) {
        Ok(result) => result,
        Err(e) => {
            println_debug!("{:#?}", e);
            exit(1);
        }
    };

    let mut conn = match pool.get_conn() {
        Ok(result) => result,
        Err(e) => {
            println_debug!("{:#?}", e);
            exit(1);
        }
    };

    conn
});

pub fn store_block(block: &Block) {
    match unsafe { CONN.exec_drop(
        "INSERT INTO block(
            timestamp,
            parent_block_hash,
            forger,
            forger_signature,
            forger_pub_key
        )
        VALUES(
            :timestamp,
            :parent_block_hash,
            :forger,
            :forger_signature,
            :forger_pub_key
        );", 
        params!{
            "timestamp" => block.timestamp.clone(),
            "parent_block_hash" => block.parent_block_hash.clone(),
            "forger" => block.forger.clone(),
            "forger_signature" => block.forger_signature.clone(),
            "forger_pub_key" => block.forger_pub_key.clone()
        }
    ) } {
        Ok(_) => {},
        Err(e) => {
            println_debug!("{:#?}", e);
            return;
        }
    }


}