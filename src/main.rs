extern crate amp;
use amp::Application;
use amp::Error;
use std::env;

static USAGE: &'static str = "\
Usage:
  amp --help       Show this help message.
  amp <directory>  Launch Amp in the given directory.
  amp <file>       Launch Amp to edit the given file.";

enum Action {
    Help,
    Run {
        paths: Vec<String>,
        options: Vec<(String, String)>,
    },
}

fn main() {
    let action = parse_args(env::args().skip(1));

    match action {
        Action::Help => println!("{}", USAGE),
        Action::Run { paths, options: _, } =>
            if let Err(e) = Application::new(&paths).and_then(|mut app| app.run()) {
                handle_error(&e)
            },
    }
}

fn parse_args(args: impl Iterator<Item = String>) -> Action {
    let mut paths = Vec::new();
    let mut options = Vec::new();
    let mut curr_option = None;

    for (i, arg) in args.enumerate() {
        if i == 0 && arg == "--help" {
            return Action::Help;
        }

        if let Some(opt_k) = curr_option.take() {
            options.push((opt_k, arg));
        } else if arg.starts_with("--") {
            curr_option = Some(arg[2..].to_string());
        } else {
            paths.push(arg);
        }
    }

    Action::Run { paths, options }
}

fn handle_error(error: &Error) {
    // Print the proximate/contextual error.
    eprintln!("error: {}", error);

    // Print the chain of other errors that led to the proximate error.
    for e in error.iter().skip(1) {
        eprintln!("caused by: {}", e);
    }

    // Print the backtrace, if available.
    if let Some(backtrace) = error.backtrace() {
        eprintln!("backtrace: {:?}", backtrace);
    }

    // Exit with an error code.
    ::std::process::exit(1);
}
