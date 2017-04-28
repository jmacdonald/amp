#[macro_use]
extern crate syntex_syntax as syntax;

use std::env;
use std::path::Path;
use std::fs::File;
use std::io::{Read, Write};
use syntax::parse::{parse_crate_from_file, ParseSess};
use syntax::ast::{ItemKind, Visibility};
use syntax::symbol::Symbol;

fn main() {
    let current_dir = env::current_dir().expect("Couldn't get the current directory");
    let mut output = File::create("src/models/application/modes/command/generated_commands")
        .expect("Couldn't create output file");
    output.write("{\nlet mut commands: HashMap<&'static str, Command> = HashMap::new();\n"
                     .as_bytes());

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

    if let &ItemKind::Mod(ref module) = command_module {
        for module_item in module.items.iter() {
            if let ItemKind::Mod(ref submodule) = module_item.node {
                for submodule_item in submodule.items.iter() {
                    if submodule_item.node.descriptive_variant() == "function" &&
                       submodule_item.vis == Visibility::Public {
                        output.write(format!("commands.insert(\"{}::{}\", {}::{});\n",
                                             module_item.ident.name.as_str(),
                                             submodule_item.ident.name.as_str(),
                                             module_item.ident.name.as_str(),
                                             submodule_item.ident.name.as_str())
                                             .as_bytes());
                    }
                }
            }
        }
    }

    output.write("commands\n}\n".as_bytes());
}
