use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn init() {
    thread::spawn(|| {
        loop {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_millis() as u64;
            let next_voting_slot = now - (now % 60000) + 60000;
            let sleep_timer = next_voting_slot - now;

            thread::sleep(Duration::from_millis(sleep_timer));
            println_debug!("{}", SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_millis() as u64);
        }
    });
}