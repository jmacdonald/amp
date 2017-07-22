extern crate amp;
use amp::Application;
use amp::Error;

fn main() {
    // Instantiate the application.
    let mut application = match Application::new() {
        Err(e) => return handle_error(&e),
        Ok(a) => a,
    };

    // Run the main application loop.
    if let Err(e) = application.run() {
        handle_error(&e)
    }
}

fn handle_error(error: &Error) {
    // Print the proximate/contextual error.
    println!("error: {}", error);

    // Print the chain of other errors that led to the proximate error.
    for e in error.iter().skip(1) {
        println!("caused by: {}", e);
    }

    // Print the backtrace, if available.
    if let Some(backtrace) = error.backtrace() {
        println!("backtrace: {:?}", backtrace);
    }

    // Exit with an error code.
    ::std::process::exit(1);
}
