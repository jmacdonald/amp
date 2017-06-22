extern crate syntex_syntax as syntax;

use std::env;
use std::fs::File;
use std::io::Write;
use syntax::parse::{parse_crate_from_file, ParseSess};
use syntax::ast::{ItemKind, Visibility};

fn main() {
    generate_commands();
}

/// This build task generates a Rust snippet which, when included later on in
/// build process, adds logic to construct a HashMap<String, Command> for all
/// public commands declared in the commands module. This facilitates runtime
/// command referencing via string, which is required for command mode, as well
/// as user-defined keymaps.
fn generate_commands() {
    // Create the output file and write the opening lines.
    let current_dir = env::current_dir().expect("Couldn't get the current directory");
    let mut output = File::create("src/commands/hash_map")
        .expect("Couldn't create output file");
    output
        .write("{\n    let mut commands: HashMap<&'static str, Command> = HashMap::new();\n"
                   .as_bytes())
        .expect("Failed to write command hash init");

    // Parse the crate and get a reference to the command module.
    let session = ParseSess::new();
    let parsed_crate =
        parse_crate_from_file(&current_dir.join("src/lib.rs"), &session)
        .expect("Couldn't parse amp crate");
    let ref command_module = parsed_crate
        .module
        .items
        .iter()
        .find(|m| m.ident.name.as_str().starts_with("command"))
        .expect("Couldn't find command module")
        .node;

    // Locate any public methods under the command module
    // and generate String => Command map entries for them.
    if let &ItemKind::Mod(ref module) = command_module {
        for module_item in module.items.iter() {
            if let ItemKind::Mod(ref submodule) = module_item.node {
                for submodule_item in submodule.items.iter() {
                    if submodule_item.node.descriptive_variant() == "function" &&
                       submodule_item.vis == Visibility::Public {
                        output
                            .write(format!("    commands.insert(\"{}::{}\", {}::{});\n",
                                           module_item.ident.name.as_str(),
                                           submodule_item.ident.name.as_str(),
                                           module_item.ident.name.as_str(),
                                           submodule_item.ident.name.as_str())
                                           .as_bytes())
                            .expect("Failed to write command");
                    }
                }
            }
        }
    }

    // Finalize the output file.
    output
        .write("    commands\n}\n".as_bytes())
        .expect("Failed to write command hash return");
}
