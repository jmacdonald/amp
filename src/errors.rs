// Create the Error, ErrorKind, ResultExt, and Result types
error_chain! {
    foreign_links {
        Io(::std::io::Error) #[cfg(unix)];
    }
}

pub static SEARCH_QUERY_MISSING: &str = "No search query";
pub static NO_SEARCH_RESULTS: &str = "No search results available";
pub static BUFFER_MISSING: &str = "No buffer available";
pub static BUFFER_PATH_MISSING: &str = "No path found for the current buffer";
pub static CURRENT_LINE_MISSING: &str = "The current line couldn't be found in the buffer";
pub static SCROLL_TO_CURSOR_FAILED: &str = "Failed to scroll to cursor position";
