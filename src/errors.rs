// Create the Error, ErrorKind, ResultExt, and Result types
error_chain! {
    foreign_links {
        Io(::std::io::Error) #[cfg(unix)];
    }
}

pub const BUFFER_MISSING: &str = "No buffer available";
pub const BUFFER_PARSE_FAILED: &str = "Failed to parse buffer";
pub const BUFFER_PATH_MISSING: &str = "No path found for the current buffer";
pub const BUFFER_RELOAD_FAILED: &str = "Unable to reload buffer";
pub const BUFFER_SAVE_FAILED: &str = "Unable to save buffer";
pub const BUFFER_SYNTAX_UPDATE_FAILED: &str = "Failed to update buffer syntax definition";
pub const BUFFER_TOKENS_FAILED: &str = "Failed to generate buffer tokens";
pub const CURRENT_LINE_MISSING: &str = "The current line couldn't be found in the buffer";
pub const FORMAT_TOOL_MISSING: &str = "No format tool configured for this filetype";
pub const LOCK_POISONED: &str = "Lock has been poisoned";
pub const NO_SEARCH_RESULTS: &str = "No search results available";
pub const SCROLL_TO_CURSOR_FAILED: &str = "Failed to scroll to cursor position";
pub const SEARCH_QUERY_MISSING: &str = "No search query";
pub const STDOUT_FAILED: &str = "Failed to acquire stdout";
pub const WORKSPACE_INIT_FAILED: &str = "Failed to initialize workspace";
