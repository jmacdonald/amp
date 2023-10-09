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
pub static BUFFER_TOKENS_FAILED: &str = "Failed to generate buffer tokens";
pub static BUFFER_PARSE_FAILED: &str = "Failed to parse buffer";
pub static BUFFER_SYNTAX_UPDATE_FAILED: &str = "Failed to update buffer syntax definition";
pub static CURRENT_LINE_MISSING: &str = "The current line couldn't be found in the buffer";
pub static SCROLL_TO_CURSOR_FAILED: &str = "Failed to scroll to cursor position";
pub static LOCK_FAILED: &str = "Failed to acquire lock";
pub static LOCK_POISONED: &str = "Lock has been poisoned";
pub static WORKSPACE_INIT_FAILED: &str = "Failed to initialize workspace";
