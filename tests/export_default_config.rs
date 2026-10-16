use std::fs;
use std::path::Path;
use std::process::ExitCode;
use trident::config::Config;

const START: &str = "<!-- DEFAULT-CONFIG-START -->";
const END: &str = "<!-- DEFAULT_CONFIG-END -->";

/// verifies that the example config in README.md is valid
fn main() -> ExitCode {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("README.md");
    let readme = fs::read_to_string(&path).expect("read README.md");
    let (_, rest) = readme
        .split_once(START)
        .expect("DEFAULT-CONFIG-START marker");
    let (block, _) = rest.split_once(END).expect("DEFAULT_CONFIG-END marker");
    let json = block
        .trim()
        .trim_start_matches("```jsonc")
        .trim_end_matches("```");
    match Config::parse(json, "README.md") {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            err.print();
            ExitCode::FAILURE
        }
    }
}
