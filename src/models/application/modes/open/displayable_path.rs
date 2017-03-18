use std::fmt;
use std::path::PathBuf;

// Newtype to make a standard path buffer presentable (via the Display
// trait), which is required for any type used in search/select mode.
pub struct DisplayablePath(pub PathBuf);

impl fmt::Display for DisplayablePath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &DisplayablePath(ref path) = self;
        write!(f, "{}", path.to_string_lossy())
    }
}
