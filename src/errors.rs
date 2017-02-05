// Create the Error, ErrorKind, ResultExt, and Result types
error_chain! {
    foreign_links {
        Io(::std::io::Error) #[cfg(unix)];
    }
}

pub static BUFFER_MISSING: &'static str = "No buffer available";
pub static BUFFER_PATH_MISSING: &'static str = "No path found for the current buffer";
pub static CURRENT_LINE_MISSING: &'static str = "The current line couldn't be found in the buffer";
pub static SCROLL_TO_CURSOR_FAILED: &'static str = "Failed to scroll to cursor position";
pub static NO_SEARCH_RESULTS: &'static str = "No matches found";
