use regex::Regex;
use std::env;
use std::fs::{self, File, read_to_string};
use std::io::Write;
use std::path::Path;
use std::result::Result;

const COMMAND_REGEX: &str =
    r"pub fn (.*)\(app: &mut Application\) -> Result";

fn main() {
    generate_commands();
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
        format!("Couldn't create output file: {}", out_file.to_string_lossy())
    })?;
    file
        .write("{\n    let mut commands: HashMap<&'static str, Command> = HashMap::new();\n"
                   .as_bytes())
        .map_err(|_| "Failed to write command hash init")?;

    Ok(file)
}

fn write_commands(output: &mut File) -> Result<(), &str> {
    let expression = Regex::new(COMMAND_REGEX)
        .expect("Failed to compile command matching regex");
    let entries = fs::read_dir("./src/commands/")
        .map_err(|_| "Failed to read command module directory")?;
    for entry in entries {
        let path = entry
            .map_err(|_| "Failed to read command module directory entry")?.path();
        let module_name = module_name(&path).unwrap();
        let content = read_to_string(&path)
            .map_err(|_| "Failed to read command module data")?;
        for captures in expression.captures_iter(&content) {
            let function_name = captures.get(1).unwrap().as_str();
            write_command(output, &module_name, function_name)?;
        }
    }

    Ok(())
}

fn write_command(output: &mut File, module_name: &str, function_name: &str) -> Result<usize, &'static str> {
    output.write(
        format!(
            "    commands.insert(\"{}::{}\", {}::{});\n",
            module_name,
            function_name,
            module_name,
            function_name
        ).as_bytes()
    ).map_err(|_| "Failed to write command")
}

fn finalize_output_file(output: &mut File) -> Result<usize, &str> {
    output.write("    commands\n}\n".as_bytes())
        .map_err(|_| "Failed to write command hash return")
}

fn module_name(path: &Path) -> Result<String, &str> {
    path.file_name().and_then(|name| {
        name.to_string_lossy().split('.').next().map(|n| n.to_string())
    }).ok_or("Unable to parse command module from file name")
}
