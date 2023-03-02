#[macro_export]
macro_rules! println_debug {
    () => (print!("\n"));
    ($($arg:tt)*) => ({
        use std::{time::{SystemTime, UNIX_EPOCH}, format_args};
        let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let timestamp = duration.as_secs();
        println!("[{}] [{}:{}] {}", timestamp, file!(), line!(), format_args!($($arg)*));
    })
}