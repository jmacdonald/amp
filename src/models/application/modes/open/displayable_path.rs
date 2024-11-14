use fragment::matching::AsStr;
use std::fmt;
use std::path::PathBuf;

// Newtype to make a standard path buffer presentable (via the Display
// trait), which is required for any type used in search/select mode.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct DisplayablePath(pub PathBuf);

impl fmt::Display for DisplayablePath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let DisplayablePath(path) = self;
        write!(f, "{}", path.to_string_lossy())
    }
}

impl AsStr for DisplayablePath {
    fn as_str(&self) -> &str {
        let DisplayablePath(path) = self;
        path.to_str().unwrap_or("")
    }
}
