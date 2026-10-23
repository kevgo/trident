use itertools::Itertools;
use std::fmt::Write;
use std::fs;
use std::path::Path;
use trident::domain::StackType;
use trident::stacks;

const START: &str = "<!-- SUPPORTED-STACKS-START -->";
const END: &str = "<!-- SUPPORTED-STACKS-END -->";
const CUSTOM_COMMAND: &str = "(your custom command)";

/// regenerates the "Supported stacks" table in README.md
#[test]
fn update_readme_stacks() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("README.md");
    let readme = fs::read_to_string(&path).expect("read README.md");
    let (before, rest) = readme
        .split_once(START)
        .expect("SUPPORTED-STACKS-START marker");
    let (_, after) = rest.split_once(END).expect("SUPPORTED-STACKS-END marker");
    let new_readme = format!("{before}{START}\n\n{}\n{END}{after}", table());
    fs::write(&path, new_readme).expect("write README.md");
}

/// provides a Markdown table listing all stacks and their lints and fixes
fn table() -> String {
    let mut result = String::from("| Stack | Linter | Fix |\n| --- | --- | --- |\n");
    for stack in stacks::all() {
        if stack.stack_type() == StackType::Unknown {
            continue;
        }
        let (lints, fixes) = if stack.stack_type() == StackType::Rust {
            (CUSTOM_COMMAND.to_string(), CUSTOM_COMMAND.to_string())
        } else {
            (
                stack.lints().iter().map(ToString::to_string).join(", "),
                stack.fixes().iter().map(ToString::to_string).join(", "),
            )
        };
        writeln!(result, "| {stack} | {lints} | {fixes} |").unwrap();
    }
    result
}
