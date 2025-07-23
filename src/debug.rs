#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        {
            use std::thread;
            eprintln!("[{:?}] {}", thread::current().id(), format_args!($($arg)*));
        }
    };
}
