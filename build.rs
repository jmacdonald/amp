use regex::Regex;
use std::env;
use std::fs::{self, read_to_string, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::result::Result;

const COMMAND_REGEX: &str = r"pub fn (.*)\(app: &mut Application\) -> Result";
const APP_SYNTAX_DIR: &str = "syntaxes";
const APP_SYNTAX_SOURCE: &str = "app_syntaxes.rs";

fn main() {
    generate_commands();
    bake_app_syntaxes();
    set_build_revision();
}

/// This build task generates a Rust snippet which, when included later on in
/// build process, adds logic to construct a HashMap<String, Command> for all
/// public commands declared in the commands module. This facilitates runtime
/// command referencing via string, which is required for command mode, as well
/// as user-defined keymaps.
fn generate_commands() {
    let mut output = create_output_file().unwrap();
    write_commands(&mut output).unwrap();
    finalize_output_file(&mut output).unwrap();
}

fn create_output_file() -> Result<File, String> {
    let out_dir = env::var("OUT_DIR").expect("The compiler did not provide $OUT_DIR");
    let out_file: std::path::PathBuf = [&out_dir, "hash_map"].iter().collect();
    let mut file = File::create(&out_file).map_err(|_| {
        format!(
            "Couldn't create output file: {}",
            out_file.to_string_lossy()
        )
    })?;
    file.write(
        "{\n    let mut commands: HashMap<&'static str, Command> = HashMap::new();\n".as_bytes(),
    )
    .map_err(|_| "Failed to write command hash init")?;

    Ok(file)
}

fn write_commands(output: &mut File) -> Result<(), &str> {
    let expression = Regex::new(COMMAND_REGEX).expect("Failed to compile command matching regex");
    let entries =
        fs::read_dir("./src/commands/").map_err(|_| "Failed to read command module directory")?;
    for entry in entries {
        let path = entry
            .map_err(|_| "Failed to read command module directory entry")?
            .path();
        let module_name = module_name(&path).unwrap();
        let content = read_to_string(&path).map_err(|_| "Failed to read command module data")?;
        for captures in expression.captures_iter(&content) {
            let function_name = captures.get(1).unwrap().as_str();
            write_command(output, &module_name, function_name)?;
        }
    }

    Ok(())
}

fn write_command(
    output: &mut File,
    module_name: &str,
    function_name: &str,
) -> Result<usize, &'static str> {
    output
        .write(
            format!(
                "    commands.insert(\"{module_name}::{function_name}\", {module_name}::{function_name});\n"
            )
            .as_bytes(),
        )
        .map_err(|_| "Failed to write command")
}

fn finalize_output_file(output: &mut File) -> Result<usize, &str> {
    output
        .write("    commands\n}\n".as_bytes())
        .map_err(|_| "Failed to write command hash return")
}

fn module_name(path: &Path) -> Result<String, &str> {
    path.file_name()
        .and_then(|name| {
            name.to_string_lossy()
                .split('.')
                .next()
                .map(|n| n.to_string())
        })
        .ok_or("Unable to parse command module from file name")
}

fn set_build_revision() {
    // Skip if the environment variable is already set
    let revision = env::var("BUILD_REVISION");
    if revision.map(|r| !r.is_empty()) == Ok(true) {
        return;
    }

    // Run the Git command to get the current commit hash
    let output = Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output()
        .expect("Failed to execute git command");

    // Parse the hash
    let build_revision = String::from_utf8(output.stdout).expect("Invalid UTF-8 sequence");

    // Write the hash as an environment variable
    println!("cargo:rustc-env=BUILD_REVISION={}", build_revision.trim());
}

fn bake_app_syntaxes() {
    println!("cargo:rerun-if-changed={APP_SYNTAX_DIR}");

    let out_dir = env::var("OUT_DIR").expect("The compiler did not provide $OUT_DIR");
    let output_path = PathBuf::from(out_dir).join(APP_SYNTAX_SOURCE);
    let syntax_dir = Path::new(APP_SYNTAX_DIR);
    let mut output = File::create(&output_path).expect("Failed to create bundled syntax source");
    output
        .write_all(b"{\n    let mut syntaxes = Vec::new();\n")
        .expect("Failed to start bundled syntax source");

    if syntax_dir.exists() {
        for syntax_path in syntax_files(syntax_dir).expect("Failed to enumerate bundled syntaxes") {
            write_syntax_loader(&mut output, &syntax_path)
                .expect("Failed to write bundled syntax source");
        }
    }

    output
        .write_all(b"    Ok(syntaxes)\n}\n")
        .expect("Failed to finish bundled syntax source");
}

fn syntax_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut paths = Vec::new();

    for entry in fs::read_dir(root).map_err(|_| "Failed to read bundled syntax directory")? {
        let path = entry
            .map_err(|_| "Failed to read bundled syntax directory entry")?
            .path();

        if path.is_dir() {
            paths.extend(syntax_files(&path)?);
        } else if path.extension().map(|ext| ext == "sublime-syntax") == Some(true) {
            paths.push(path);
        }
    }

    paths.sort();
    Ok(paths)
}

fn write_syntax_loader(output: &mut File, syntax_path: &Path) -> Result<usize, &'static str> {
    let syntax_path = syntax_path
        .canonicalize()
        .map_err(|_| "Failed to canonicalize bundled syntax path")?;

    output
        .write(
            format!(
                "    syntaxes.push(syntect::parsing::SyntaxDefinition::load_from_str(include_str!({syntax_path:?}), true, None)?);\n"
            )
            .as_bytes(),
        )
        .map_err(|_| "Failed to write bundled syntax source")
}
