use schemars::JsonSchema;
use serde_with::DeserializeFromStr;
use std::fmt::Display;
use strum::EnumString;

#[derive(Clone, Copy, Debug, DeserializeFromStr, JsonSchema, PartialEq, Eq, Hash, EnumString)]
#[strum(ascii_case_insensitive)]
#[schemars(rename_all = "lowercase")]
pub enum StackType {
    Css,
    Cucumber,
    Dockerfile,
    Go,
    Java,
    Json,
    JsonC,
    Markdown,
    Python,
    Rust,
    Shell,
    Sql,
    Toml,
    Typescript,
    Unknown,
    Yml,
}

impl Display for StackType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StackType::Css => f.write_str("CSS"),
            StackType::Cucumber => f.write_str("Cucumber"),
            StackType::Dockerfile => f.write_str("Dockerfile"),
            StackType::Go => f.write_str("Go"),
            StackType::Java => f.write_str("Java"),
            StackType::Json => f.write_str("JSON"),
            StackType::JsonC => f.write_str("JSONC"),
            StackType::Markdown => f.write_str("Markdown"),
            StackType::Python => f.write_str("Python"),
            StackType::Rust => f.write_str("Rust"),
            StackType::Shell => f.write_str("Shell"),
            StackType::Sql => f.write_str("SQL"),
            StackType::Toml => f.write_str("TOML"),
            StackType::Typescript => f.write_str("TypeScript"),
            StackType::Unknown => f.write_str("other"),
            StackType::Yml => f.write_str("YML"),
        }
    }
}
