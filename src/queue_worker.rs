use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::blockchain;

use super::transaction_queue;

pub fn init() {
    thread::spawn(|| {
        loop {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_millis() as u64;
            let next_voting_slot = now - (now % 60000) + 60000;
            let sleep_timer = next_voting_slot - now;

            thread::sleep(Duration::from_millis(sleep_timer));

            let transactions_for_block: Vec<blockchain::Tx>;
            unsafe { 
                let mut transactions: Vec<blockchain::Tx> = transaction_queue::TX_HASHMAP.as_mut().unwrap().values().cloned().collect();
                transactions.sort_unstable_by(|a, b| b.network_fee.cmp(&a.network_fee));
                transactions_for_block= transactions.into_iter().take(100).collect();
                println_debug!("{:#?}", transactions_for_block);
            };

            let block = blockchain::Block::new(transactions_for_block);
            if block.timestamp == 0 {
                println_debug!("Block could not be created, skipping current iteration!");
                continue;
            }

            println_debug!("{:#?}", block);
        }
    });
}