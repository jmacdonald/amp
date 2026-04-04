use scribe::buffer::Token;
use scribe::{Buffer, Workspace};
use std::fs;
use std::path::{Path, PathBuf};
use syntect::parsing::{SyntaxSet, SyntaxSetBuilder};
use yaml_rust::{Yaml, YamlLoader};

const SYNTAX_FIXTURE_ROOT: &str = "tests/fixtures/syntaxes";
const EXPECTATIONS_SUFFIX: &str = ".tokens.yml";
const TOKEN_DEBUG_WINDOW_SIZE: usize = 12;

// A single token expected to be emitted by the syntax engine: the lexeme text
// plus the most specific scope name associated with it.
#[derive(Debug, PartialEq)]
struct TokenExpectation {
    text: String,
    scope: String,
}

// Everything needed to run one syntax fixture: a display name for failures, the
// buffer path used to select the syntax, the fixture file to tokenize, and the
// ordered expectations to check against the resulting token stream.
#[derive(Debug)]
struct SyntaxFixture {
    name: String,
    buffer_path: PathBuf,
    fixture_path: PathBuf,
    expectations: Vec<TokenExpectation>,
}

#[test]
fn bundled_syntax_definitions_load() {
    build_syntax_set();
}

#[test]
// Loads every syntax fixture and checks that each expected token/scope pair
// appears in order in the actual token stream.
fn syntax_fixtures_tokenize_expected_scopes() {
    let syntax_set = build_syntax_set();
    let fixtures = load_syntax_fixtures(Path::new(SYNTAX_FIXTURE_ROOT));

    assert!(
        !fixtures.is_empty(),
        "no syntax fixtures found in {SYNTAX_FIXTURE_ROOT}"
    );

    for fixture in fixtures {
        let actual_tokens = tokenize_fixture(&syntax_set, &fixture);
        assert_token_subsequence(&fixture, &actual_tokens);
    }
}

// Builds a syntax set from the repository's bundled syntax definitions.
fn build_syntax_set() -> SyntaxSet {
    let mut builder = SyntaxSetBuilder::new();
    builder
        .add_from_folder("syntaxes", true)
        .expect("failed to load bundled syntax definitions");
    builder.build()
}

// Loads every expectations file under the syntax fixtures root and turns each
// one into a fully populated SyntaxFixture.
fn load_syntax_fixtures(root: &Path) -> Vec<SyntaxFixture> {
    let mut expectations_paths = Vec::new();
    collect_expectations_paths(root, &mut expectations_paths);
    expectations_paths.sort();

    expectations_paths
        .into_iter()
        .map(|path| load_syntax_fixture(&path))
        .collect()
}

// Walks the fixture tree recursively and collects every expectations file.
fn collect_expectations_paths(root: &Path, paths: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(root).unwrap_or_else(|error| {
        panic!(
            "failed to read fixture directory {}: {error}",
            root.display()
        )
    });

    for entry in entries {
        let path = entry
            .unwrap_or_else(|error| {
                panic!(
                    "failed to read fixture entry in {}: {error}",
                    root.display()
                )
            })
            .path();

        if path.is_dir() {
            collect_expectations_paths(&path, paths);
        } else if path.to_string_lossy().ends_with(EXPECTATIONS_SUFFIX) {
            paths.push(path);
        }
    }
}

// Parses one expectations file, validates its basic shape, and joins it with
// the fixture file it describes.
fn load_syntax_fixture(expectations_path: &Path) -> SyntaxFixture {
    let fixture_path = PathBuf::from(
        expectations_path
            .to_string_lossy()
            .strip_suffix(EXPECTATIONS_SUFFIX)
            .unwrap_or_else(|| {
                panic!(
                    "invalid expectations suffix for {}",
                    expectations_path.display()
                )
            }),
    );
    let expectations_document = fs::read_to_string(expectations_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", expectations_path.display()));
    let yaml = YamlLoader::load_from_str(&expectations_document)
        .unwrap_or_else(|error| panic!("failed to parse {}: {error}", expectations_path.display()));
    let root = yaml
        .first()
        .unwrap_or_else(|| panic!("missing YAML document in {}", expectations_path.display()));

    let buffer_path = required_string(root, "buffer_path", expectations_path);
    let expectations = required_array(root, "expectations", expectations_path)
        .iter()
        .map(|entry| TokenExpectation {
            text: required_string(entry, "text", expectations_path).to_string(),
            scope: required_string(entry, "scope", expectations_path).to_string(),
        })
        .collect::<Vec<_>>();

    assert!(
        fixture_path.exists(),
        "fixture file {} is missing for {}",
        fixture_path.display(),
        expectations_path.display()
    );
    assert!(
        !expectations.is_empty(),
        "fixture {} must include at least one expectation",
        expectations_path.display()
    );

    SyntaxFixture {
        name: expectations_path
            .strip_prefix(SYNTAX_FIXTURE_ROOT)
            .unwrap_or(expectations_path)
            .display()
            .to_string(),
        buffer_path: PathBuf::from(buffer_path),
        fixture_path,
        expectations,
    }
}

// Reads a required string key from a YAML mapping and fails with fixture
// context if the key is missing or not a string.
fn required_string<'a>(yaml: &'a Yaml, key: &str, path: &Path) -> &'a str {
    yaml[key]
        .as_str()
        .unwrap_or_else(|| panic!("missing string key '{key}' in {}", path.display()))
}

// Reads a required array key from a YAML mapping and fails with fixture
// context if the key is missing or not an array.
fn required_array<'a>(yaml: &'a Yaml, key: &str, path: &Path) -> &'a Vec<Yaml> {
    yaml[key]
        .as_vec()
        .unwrap_or_else(|| panic!("missing array key '{key}' in {}", path.display()))
}

// Opens the fixture file in a scratch workspace buffer, lets the syntax
// engine tokenize it, and flattens the result to text/scope pairs that are easy
// to assert against in tests.
fn tokenize_fixture(syntax_set: &SyntaxSet, fixture: &SyntaxFixture) -> Vec<TokenExpectation> {
    let mut workspace = Workspace::with_syntax_set(Path::new("."), syntax_set.clone())
        .expect("workspace init failed");
    let fixture_contents = fs::read_to_string(&fixture.fixture_path).unwrap_or_else(|error| {
        panic!("failed to read {}: {error}", fixture.fixture_path.display())
    });

    let mut buffer = Buffer::new();
    buffer.path = Some(fixture.buffer_path.clone());
    buffer.insert(fixture_contents);
    workspace.add_buffer(buffer);

    let syntax_name = workspace
        .current_buffer
        .as_ref()
        .and_then(|buffer| buffer.syntax_definition.as_ref())
        .map(|definition| definition.name.as_str())
        .unwrap_or("<missing syntax>");

    let token_set = workspace.current_buffer_tokens().unwrap_or_else(|error| {
        panic!(
            "failed to tokenize fixture {} with syntax {}: {error}",
            fixture.name, syntax_name
        )
    });
    let mut iterator = token_set.iter().unwrap_or_else(|error| {
        panic!(
            "failed to iterate tokens for fixture {} with syntax {}: {error}",
            fixture.name, syntax_name
        )
    });
    let tokens = iterator
        .by_ref()
        .filter_map(|token| match token {
            Token::Lexeme(lexeme) => Some(TokenExpectation {
                text: lexeme.value.to_string(),
                scope: format!("{}", lexeme.scope)
                    .split_whitespace()
                    .last()
                    .unwrap_or_default()
                    .to_string(),
            }),
            Token::Newline => None,
        })
        .collect::<Vec<_>>();

    if let Some(error) = iterator.error {
        panic!(
            "token iterator failed for fixture {} with syntax {}: {error}",
            fixture.name, syntax_name
        );
    }

    tokens
}

// Verifies that the expectations appear in order within the token stream
// without requiring the fixture to spell out every intermediate token.
fn assert_token_subsequence(fixture: &SyntaxFixture, actual_tokens: &[TokenExpectation]) {
    let mut start_index = 0;

    for expectation in &fixture.expectations {
        if let Some(found_index) = actual_tokens[start_index..]
            .iter()
            .position(|token| token == expectation)
        {
            start_index += found_index + 1;
            continue;
        }

        panic!(
            "fixture {} missing expected token {:?}\nactual tokens near search start:\n{}",
            fixture.name,
            expectation,
            format_token_window(actual_tokens, start_index)
        );
    }
}

fn format_token_window(tokens: &[TokenExpectation], start_index: usize) -> String {
    // Show a short slice of nearby tokens so failures stay readable.
    let end_index = usize::min(tokens.len(), start_index + TOKEN_DEBUG_WINDOW_SIZE);
    tokens[start_index..end_index]
        .iter()
        .enumerate()
        .map(|(offset, token)| {
            format!(
                "{}: {:?} -> {}",
                start_index + offset,
                token.text,
                token.scope
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
