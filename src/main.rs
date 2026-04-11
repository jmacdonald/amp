use amp::Application;
use amp::Error;
use log::debug;
use std::backtrace::BacktraceStatus;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Initialize logging
    env_logger::init();

    // Instantiate, run, and handle errors for the application.
    match Application::new(&args) {
        Ok(mut app) => {
            if let Err(error) = app.run() {
                app.view.shutdown();
                handle_error(&error)
            }
        }
        Err(error) => handle_error(&error),
    }

    debug!("exiting");
}

fn handle_error(error: &Error) {
    // Print the proximate/contextual error.
    eprintln!("error: {error}");

    // Print the chain of other errors that led to the proximate error.
    for e in error.chain().skip(1) {
        eprintln!("caused by: {e}");
    }

    // Print the backtrace in a readable, frame-per-line format when it exists.
    let backtrace = error.backtrace();
    if backtrace.status() == BacktraceStatus::Captured {
        eprintln!("backtrace:\n{backtrace}");
    }

    // Exit with an error code.
    ::std::process::exit(1);
}
