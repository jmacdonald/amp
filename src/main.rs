use amp::Application;
use amp::Error;
use std::env;

#[macro_use]
mod debug;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Instantiate, run, and handle errors for the application.
    if let Some(e) = Application::new(&args).and_then(|mut app| app.run()).err() {
        handle_error(&e)
    }

    debug_log!("[main] exiting");
}

fn handle_error(error: &Error) {
    // Print the proximate/contextual error.
    eprintln!("error: {error}");

    // Print the chain of other errors that led to the proximate error.
    for e in error.iter().skip(1) {
        eprintln!("caused by: {e}");
    }

    // Print the backtrace, if available.
    if let Some(backtrace) = error.backtrace() {
        eprintln!("backtrace: {backtrace:?}");
    }

    // Exit with an error code.
    ::std::process::exit(1);
}
