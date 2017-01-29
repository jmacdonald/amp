extern crate amp;
use amp::Application;

fn main() {
    if let Err(ref e) = Application::run() {
        // Print the proximate/contextual error.
        println!("error: {}", e);

        // Print the chain of other errors that led to the proximate error.
        for e in e.iter().skip(1) {
            println!("caused by: {}", e);
        }

        // Print the backtrace, if available.
        if let Some(backtrace) = e.backtrace() {
            println!("backtrace: {:?}", backtrace);
        }

        // Exit with an error code.
        ::std::process::exit(1);
    }
}
