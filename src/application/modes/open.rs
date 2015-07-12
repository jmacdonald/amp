extern crate bloodhound;

use std::path::PathBuf;
use self::bloodhound::index::Index;
use self::bloodhound::matching::Result;

pub struct OpenMode {
    pub input: String,
    pub index: Index,
    pub results: Option<Vec<Result>>,
    pub selected_result_index: usize,
}

pub fn new(path: PathBuf) -> OpenMode {
    // Build and populate the index.
    let mut index = bloodhound::index::new(path);
    index.populate();

    OpenMode{
        input: String::new(),
        index: index,
        results: None,
        selected_result_index: 0
    }
}
