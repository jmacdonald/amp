use crate::errors::*;
use std::path::PathBuf;
use syntect::dumps::from_uncompressed_data;
use syntect::parsing::SyntaxSet;

pub struct SyntaxLoader {
    path: PathBuf,
}

impl SyntaxLoader {
    pub fn new(path: PathBuf) -> SyntaxLoader {
        SyntaxLoader { path }
    }

    pub fn load(self) -> Result<SyntaxSet> {
        // Load syntect default + app-bundled syntax sets serialized in build.rs.
        let mut builder = from_uncompressed_data::<SyntaxSet>(include_bytes!(concat!(
            env!("OUT_DIR"),
            "/app_syntaxes.packdump"
        )))
        .context("Couldn't load bundled syntax definitions")?
        .into_builder();

        // Merge user-defined syntaxes when a user syntax directory is present.
        if self.path.is_dir() {
            builder.add_from_folder(&self.path, true)?;
        }

        Ok(builder.build())
    }
}

#[cfg(test)]
mod tests {
    use super::SyntaxLoader;
    use std::path::PathBuf;

    #[test]
    fn load_includes_bundled_and_fixture_syntaxes() {
        let syntax_set = SyntaxLoader::new(PathBuf::from("tests/fixtures/user_syntaxes"))
            .load()
            .unwrap();

        assert!(syntax_set.find_syntax_by_name("Rust").is_some());
        assert!(syntax_set.find_syntax_by_name("Amp").is_some());
    }

    #[test]
    fn load_ignores_missing_user_syntax_directory() {
        let missing_path = PathBuf::from("tests/fixtures/missing_syntaxes");
        assert!(!missing_path.exists());

        let syntax_set = SyntaxLoader::new(missing_path).load().unwrap();

        assert!(syntax_set.find_syntax_by_name("Rust").is_some());
    }
}
