use crate::domain::{Ignores, Result, StackType, Tool, UserError};
use ahash::AHashMap;
use itertools::Itertools;
use jsonc_parser::ParseOptions;
use schemars::JsonSchema;
use serde::Deserialize;
use std::fmt::Display;
use std::fs;
use std::path::Path;

/// name of the config file written by `trident init:config`
pub const FILENAME: &str = "trident.jsonc";

/// config filenames recognized by Trident, in load order
pub const CONFIG_FILENAMES: [&str; 2] = ["trident.json", FILENAME];

/// VS Code / JSON language-server schema URL for `trident.json`
pub const SCHEMA_URL: &str =
    "https://github.com/kevgo/trident/raw/refs/heads/main/docs/schema.json";

#[derive(Debug, Default, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
#[schemars(title = "Trident configuration")]
pub struct Config {
    /// JSON Schema URL for editor support
    #[serde(rename = "$schema")]
    #[schemars(rename = "$schema")]
    pub schema: Option<String>,

    // custom fixes that aren't stack-specific
    #[serde(alias = "global-fixes")]
    #[schemars(rename = "global-fixes")]
    pub global_fixes: Option<Vec<ToolDefinition>>,

    // custom lints that aren't stack-specific
    #[serde(alias = "global-lints")]
    #[schemars(rename = "global-lints")]
    pub global_lints: Option<Vec<ToolDefinition>>,

    // files that should be excluded when running any tool
    #[serde(alias = "ignore-files")]
    #[schemars(rename = "ignore-files")]
    pub ignore_files: Option<Vec<String>>,

    // application-specific configuration
    pub applications: Option<ApplicationSection>,

    // stack-specific configuration
    #[schemars(with = "Option<std::collections::BTreeMap<StackType, StackConfig>>")]
    pub stacks: Option<AHashMap<StackType, StackConfig>>,

    /// define the functional tests
    pub tests: Option<Vec<ToolDefinition>>,

    /// configure the default behavior of the Trident commands
    pub commands: Option<CommandsSection>,
}

impl Config {
    /// provides all files that should be excluded when running the given app for the given operation
    pub fn ignores_for(&self, tool: &dyn Tool, op: Operation) -> Result<Ignores> {
        let Some(app_section) = self.app_section(tool) else {
            return Ok(Ignores::empty());
        };
        let mut patterns = app_section.ignore_files().to_vec();
        if let Some(operation) = app_section.operation(op) {
            patterns.extend_from_slice(operation.ignore_files());
        }
        Ignores::new(&patterns, Path::new("./"))
    }

    /// whether the given application is enabled (missing config means enabled)
    #[must_use]
    pub fn app_enabled(&self, tool: &dyn Tool) -> bool {
        self.app_section(tool).is_none_or(Application::enabled)
    }

    /// whether the given application is enabled for the given operation
    ///
    /// An operation is enabled when the app is enabled and that operation is not `enabled: false`.
    /// Missing keys default to enabled.
    #[must_use]
    pub fn operation_enabled(&self, tool: &dyn Tool, op: Operation) -> bool {
        let Some(app_section) = self.app_section(tool) else {
            return true;
        };
        if !app_section.enabled() {
            return false;
        }
        let Some(operation) = app_section.operation(op) else {
            return true;
        };
        operation.enabled()
    }

    fn app_section(&self, tool: &dyn Tool) -> Option<&dyn Application> {
        self.applications
            .as_ref()
            .and_then(|apps_section| tool.config_section(apps_section))
    }

    pub fn load() -> Result<Self> {
        for filename in CONFIG_FILENAMES {
            match fs::read_to_string(filename) {
                Ok(text) => return Self::parse(&text, filename),
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
                Err(err) => {
                    return Err(UserError::ConfigCannotRead {
                        filename: filename.to_string(),
                        err: err.to_string(),
                    });
                }
            }
        }
        Ok(Self::default())
    }

    pub(crate) fn parse(text: &str, filename: &str) -> Result<Config> {
        // empty or comment-only files deserialize as null, hence Option
        let config: Option<Config> =
            jsonc_parser::parse_to_serde_value(text, &ParseOptions::default()).map_err(|err| {
                UserError::ConfigCannotParse {
                    filename: filename.to_string(),
                    err: err.to_string(),
                }
            })?;
        Ok(config.unwrap_or_default())
    }

    /// provides the matcher for the files that should not be linted
    pub fn ignores(&self) -> Result<Ignores> {
        Ignores::new(
            self.ignore_files.as_deref().unwrap_or_default(),
            Path::new("./"),
        )
    }

    /// provides the configuration for the given stack type
    #[must_use]
    pub fn stack_config(&self, stack_type: StackType) -> Option<&StackConfig> {
        self.stacks.as_ref()?.get(&stack_type)
    }

    /// provides keep-sorted configuration if present
    #[must_use]
    pub fn keep_sorted(&self) -> Option<&ApplicationWithFile> {
        self.applications.as_ref()?.keep_sorted.as_ref()
    }

    /// provides the tests with the given names, or all if no names are given
    pub fn select_tests(&self, requested: &[String]) -> Result<Vec<&ToolDefinition>> {
        let all_tests = self.tests.as_deref().unwrap_or_default();
        if requested.is_empty() {
            return Ok(all_tests.iter().collect());
        }
        let mut result = Vec::with_capacity(requested.len());
        let mut unknown = Vec::new();
        for name in requested.iter().unique() {
            match all_tests.iter().find(|test| test.name_or_command() == name) {
                Some(test) => result.push(test),
                None => unknown.push(name.to_owned()),
            }
        }
        if !unknown.is_empty() {
            return Err(UserError::UnknownTest {
                names: unknown,
                available: all_tests
                    .iter()
                    .map(|test| test.name_or_command().to_string())
                    .collect(),
            });
        }
        Ok(result)
    }

    /// determines the tests to run using the given --test CLI flag,
    /// the config file settings,
    /// and default tests for the command
    pub fn tests_for(
        &self,
        cli: &[String],
        command: impl FnOnce(&CommandsSection) -> Option<&CommandConfig>,
        default: DefaultTests,
    ) -> Result<Vec<&ToolDefinition>> {
        let requested = if cli.is_empty() {
            self.commands
                .as_ref()
                .and_then(command)
                .and_then(|config| config.test.as_deref())
        } else {
            Some(cli)
        };
        match requested {
            None => match default {
                DefaultTests::All => self.select_tests(&[]),
                DefaultTests::None => Ok(Vec::new()),
            },
            Some([]) => Ok(Vec::new()),
            Some(names) => self.select_tests(names),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ToolDefinition {
    /// display name for the fix
    pub name: Option<String>,

    /// the command that implements the fix
    pub command: String,
}

impl ToolDefinition {
    /// provides the name of the tool, or the command if the tool has no name
    fn name_or_command(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.command)
    }

    /// Converts this tool into an executable whose printed name matches built-in tools:
    /// `{operation} {stack} ({name})`.
    #[must_use]
    pub fn to_executable(&self, operation: Operation, stack: StackType) -> conc::Executable {
        let tool_name = self.name.as_deref().unwrap_or(&self.command);
        conc::Executable {
            name: format!("{operation} {stack} ({tool_name})"),
            command: conc::shell_command(&self.command),
        }
    }

    #[must_use]
    pub fn to_sequence(&self) -> conc::Sequence {
        conc::Sequence::one(conc::Executable {
            name: self.name_or_command().to_string(),
            command: conc::shell_command(&self.command),
        })
    }
}

/// the tests a command runs by default
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DefaultTests {
    /// run all tests defined in the config file
    All,
    /// run no tests
    None,
}

/// settings for individual Trident commands
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CommandsSection {
    /// default tests for `trident ci`
    pub ci: Option<CommandConfig>,

    /// default tests for `trident full`
    pub full: Option<CommandConfig>,

    /// default tests for `trident pitstop`
    pub pitstop: Option<CommandConfig>,

    /// default tests for `trident test`
    pub test: Option<CommandConfig>,
}

/// default tests and other settings for a Trident command
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CommandConfig {
    /// names of tests this command runs by default
    pub test: Option<Vec<String>>,
}

/// provides the configured tests with the given names as parallel `conc::Sequences`
pub(crate) fn to_sequences(tools: Vec<&ToolDefinition>) -> Vec<conc::Sequence> {
    tools.into_iter().map(ToolDefinition::to_sequence).collect()
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StackConfig {
    /// customize the lints for this stack
    pub lint: Option<StackTools>,

    /// customize the fixes for this stack
    pub fix: Option<StackTools>,
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StackTools {
    /// these commands run in addition to the built-in ones
    pub add: Option<Vec<ToolDefinition>>,

    /// these commands replace the built-in ones
    pub replace: Option<Vec<ToolDefinition>>,
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ApplicationSection {
    #[serde(alias = "actionlint")]
    #[schemars(rename = "actionlint")]
    pub actionlint: Option<ApplicationNoFile>,

    pub biome: Option<ApplicationWithFile>,

    pub checkstyle: Option<ApplicationNoFile>,

    #[serde(alias = "delete-empty-folders")]
    #[schemars(rename = "delete-empty-folders")]
    pub delete_empty_folders: Option<ApplicationNoFile>,

    pub dprint: Option<ApplicationWithFile>,

    #[serde(alias = "gherkin-lint")]
    #[schemars(rename = "gherkin-lint")]
    pub gherkin_lint: Option<ApplicationWithFile>,

    pub ghokin: Option<ApplicationWithFile>,

    #[serde(alias = "git-diff-check")]
    #[schemars(rename = "git-diff-check")]
    pub git_diff_check: Option<ApplicationNoFile>,

    pub gofumpt: Option<ApplicationWithFile>,

    #[serde(alias = "golangci-lint")]
    #[schemars(rename = "golangci-lint")]
    pub golangci_lint: Option<ApplicationNoFile>,

    pub hadolint: Option<ApplicationWithFile>,

    #[serde(alias = "keep-sorted")]
    #[schemars(rename = "keep-sorted")]
    pub keep_sorted: Option<ApplicationWithFile>,

    pub prettier: Option<ApplicationWithFile>,

    pub pyright: Option<ApplicationWithFile>,

    pub ruff: Option<ApplicationWithFile>,

    pub rumdl: Option<ApplicationWithFile>,

    pub sqlfmt: Option<ApplicationWithFile>,

    pub taplo: Option<ApplicationWithFile>,

    #[serde(alias = "text-runner")]
    #[schemars(rename = "text-runner")]
    pub text_runner: Option<ApplicationNoFile>,

    pub tikibase: Option<ApplicationNoFile>,
}

/// a Trident operation that applications can be configured for
///
/// Operations are more basic primitives than Trident commands.
/// All Trident commands execute variations of these three basic operations on various files.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Operation {
    Lint,
    Fix,
    FixUnsafe,
}

impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::Lint => f.write_str("lint"),
            Operation::Fix => f.write_str("fix"),
            Operation::FixUnsafe => f.write_str("unsafe-fix"),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Operations<T> {
    pub lint: Option<T>,

    pub fix: Option<T>,

    #[serde(alias = "fix-unsafe")]
    #[schemars(rename = "fix-unsafe")]
    pub fix_unsafe: Option<T>,
}

impl<T> Operations<T> {
    fn get(&self, op: Operation) -> Option<&T> {
        match op {
            Operation::Lint => self.lint.as_ref(),
            Operation::Fix => self.fix.as_ref(),
            Operation::FixUnsafe => self.fix_unsafe.as_ref(),
        }
    }
}

pub trait Application {
    fn enabled(&self) -> bool;

    fn ignores(&self) -> Result<Ignores> {
        Ignores::new(self.ignore_files(), Path::new("./"))
    }

    fn operation(&self, op: Operation) -> Option<&dyn Application>;

    fn ignore_files(&self) -> &[String];
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ApplicationNoFile {
    /// enable or disable the entire application
    pub enabled: Option<bool>,

    /// enable or disable individual operations
    pub operations: Option<Operations<ApplicationNoFileOperation>>,
}

impl Application for ApplicationNoFile {
    fn enabled(&self) -> bool {
        self.enabled.unwrap_or(true)
    }

    fn operation(&self, op: Operation) -> Option<&dyn Application> {
        self.operations
            .as_ref()?
            .get(op)
            .map(|cfg| cfg as &dyn Application)
    }

    fn ignore_files(&self) -> &[String] {
        &[]
    }
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ApplicationWithFile {
    /// enable or disable the entire application
    pub enabled: Option<bool>,

    /// make this app ignore these files
    #[serde(alias = "ignore-files")]
    #[schemars(rename = "ignore-files")]
    pub ignore_files: Option<Vec<String>>,

    /// enable or disable individual operations
    pub operations: Option<Operations<ApplicationWithFileOperation>>,
}

impl Application for ApplicationWithFile {
    fn enabled(&self) -> bool {
        self.enabled.unwrap_or(true)
    }

    fn operation(&self, op: Operation) -> Option<&dyn Application> {
        self.operations
            .as_ref()?
            .get(op)
            .map(|cfg| cfg as &dyn Application)
    }

    fn ignore_files(&self) -> &[String] {
        self.ignore_files.as_deref().unwrap_or_default()
    }
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ApplicationNoFileOperation {
    pub enabled: Option<bool>,
}

impl Application for ApplicationNoFileOperation {
    fn enabled(&self) -> bool {
        self.enabled.unwrap_or(true)
    }

    fn operation(&self, _op: Operation) -> Option<&dyn Application> {
        None
    }

    fn ignore_files(&self) -> &[String] {
        &[]
    }
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ApplicationWithFileOperation {
    /// enable or disable this operation
    pub enabled: Option<bool>,
    #[serde(alias = "ignore-files")]
    #[schemars(rename = "ignore-files")]

    /// files that this operation should ignore
    pub ignore_files: Option<Vec<String>>,
}

impl Application for ApplicationWithFileOperation {
    fn enabled(&self) -> bool {
        self.enabled.unwrap_or(true)
    }

    fn operation(&self, _op: Operation) -> Option<&dyn Application> {
        None
    }

    fn ignore_files(&self) -> &[String] {
        self.ignore_files.as_deref().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {

    mod parse {
        use crate::config::StackTools;
        use crate::config::{CommandConfig, CommandsSection, Config, StackConfig, ToolDefinition};
        use crate::domain::{StackType, UserError};
        use ahash::AHashMap;
        use big_s::S;

        fn stack_map(
            stack_type: StackType,
            config: StackConfig,
        ) -> AHashMap<StackType, StackConfig> {
            let mut map = AHashMap::new();
            map.insert(stack_type, config);
            map
        }

        #[test]
        fn defined() {
            let give = r#"
{
  "global-lints": [
    { "command": "lints/one.sh" },
    { "name": "custom lint 2", "command": "lints/two.sh" }
  ],
  "global-fixes": [
    { "command": "fixes/organize.py" },
    { "name": "sort alphabetically", "command": "fixes/sort.py" }
  ]
}
"#;
            let have = Config::parse(give, "test.json").unwrap();
            let want = Config {
                schema: None,
                global_fixes: Some(vec![
                    ToolDefinition {
                        name: None,
                        command: S("fixes/organize.py"),
                    },
                    ToolDefinition {
                        name: Some(S("sort alphabetically")),
                        command: S("fixes/sort.py"),
                    },
                ]),
                global_lints: Some(vec![
                    ToolDefinition {
                        name: None,
                        command: S("lints/one.sh"),
                    },
                    ToolDefinition {
                        name: Some(S("custom lint 2")),
                        command: S("lints/two.sh"),
                    },
                ]),
                ignore_files: None,
                applications: None,
                stacks: None,
                tests: None,
                commands: None,
            };
            pretty::assert_eq!(have, want);
        }

        #[test]
        fn empty() {
            let give = r#"{ "global-lints": [], "global-fixes": [] }"#;
            let have = Config::parse(give, "test.json").unwrap();
            let want = Config {
                schema: None,
                global_lints: Some(vec![]),
                global_fixes: Some(vec![]),
                ignore_files: None,
                applications: None,
                stacks: None,
                tests: None,
                commands: None,
            };
            assert_eq!(have, want);
        }

        #[test]
        fn none() {
            let have = Config::parse("", "test.json").unwrap();
            let want = Config {
                schema: None,
                global_lints: None,
                global_fixes: None,
                ignore_files: None,
                applications: None,
                stacks: None,
                tests: None,
                commands: None,
            };
            assert_eq!(have, want);
        }

        #[test]
        fn ignore() {
            let give = r#"{ "ignore-files": ["a.css", "b/"] }"#;
            let have = Config::parse(give, "test.json").unwrap();
            let want = Config {
                schema: None,
                global_lints: None,
                global_fixes: None,
                ignore_files: Some(vec![S("a.css"), S("b/")]),
                applications: None,
                stacks: None,
                tests: None,
                commands: None,
            };
            pretty::assert_eq!(have, want);
        }

        #[test]
        fn comments() {
            let give = r#"
{
  // files Trident should skip
  "ignore-files": ["a.css", "b/"]
}
"#;
            let have = Config::parse(give, "test.json").unwrap();
            let want = Config {
                schema: None,
                global_lints: None,
                global_fixes: None,
                ignore_files: Some(vec![S("a.css"), S("b/")]),
                applications: None,
                stacks: None,
                tests: None,
                commands: None,
            };
            pretty::assert_eq!(have, want);
        }

        #[test]
        fn trailing_comma() {
            let give = r#"
{
  "ignore-files": ["a.css", "b/"],
}
"#;
            let have = Config::parse(give, "test.json").unwrap();
            let want = Config {
                schema: None,
                global_lints: None,
                global_fixes: None,
                ignore_files: Some(vec![S("a.css"), S("b/")]),
                applications: None,
                stacks: None,
                tests: None,
                commands: None,
            };
            pretty::assert_eq!(have, want);
        }

        #[test]
        fn empty_or_comment_only() {
            pretty::assert_eq!(Config::parse("", "test.json").unwrap(), Config::default());
            pretty::assert_eq!(
                Config::parse("  // only a comment\n", "test.json").unwrap(),
                Config::default()
            );
        }

        #[test]
        fn stack_type_map_key_is_case_insensitive() {
            let give = r#"
{
  "stacks": {
    "PyThOn": {
      "lint": {
        "add": [{ "name": "mypy", "command": "mypy ." }]
      }
    }
  }
}
"#;
            let have = Config::parse(give, "test.json").unwrap();
            let want = Config {
                schema: None,
                global_fixes: None,
                global_lints: None,
                ignore_files: None,
                applications: None,
                stacks: Some(stack_map(
                    StackType::Python,
                    StackConfig {
                        lint: Some(StackTools {
                            add: Some(vec![ToolDefinition {
                                name: Some(S("mypy")),
                                command: S("mypy ."),
                            }]),
                            replace: None,
                        }),
                        fix: None,
                    },
                )),
                tests: None,
                commands: None,
            };
            pretty::assert_eq!(have, want);
        }

        #[test]
        fn stack_lint() {
            let give = r#"
{
  "stacks": {
    "rust": {
      "lint": {
        "replace": [{ "name": "Clippy", "command": "cargo clippy --all-targets" }]
      }
    }
  }
}
"#;
            let have = Config::parse(give, "test.json").unwrap();
            let want = Config {
                schema: None,
                global_fixes: None,
                global_lints: None,
                ignore_files: None,
                applications: None,
                stacks: Some(stack_map(
                    StackType::Rust,
                    StackConfig {
                        lint: Some(StackTools {
                            replace: Some(vec![ToolDefinition {
                                name: Some(S("Clippy")),
                                command: S("cargo clippy --all-targets"),
                            }]),
                            add: None,
                        }),
                        fix: None,
                    },
                )),
                tests: None,
                commands: None,
            };
            pretty::assert_eq!(have, want);
        }

        #[test]
        fn stack_add_lint() {
            let give = r#"
{
  "stacks": {
    "python": {
      "lint": {
        "add": [{ "name": "mypy", "command": "mypy ." }]
      }
    }
  }
}
"#;
            let have = Config::parse(give, "test.json").unwrap();
            let want = Config {
                schema: None,
                global_fixes: None,
                global_lints: None,
                ignore_files: None,
                applications: None,
                stacks: Some(stack_map(
                    StackType::Python,
                    StackConfig {
                        lint: Some(StackTools {
                            add: Some(vec![ToolDefinition {
                                name: Some(S("mypy")),
                                command: S("mypy ."),
                            }]),
                            replace: None,
                        }),
                        fix: None,
                    },
                )),
                tests: None,
                commands: None,
            };
            pretty::assert_eq!(have, want);
        }

        #[test]
        fn name_allows_underscore() {
            let give = r#"
{
  "global_lints": [
    { "name": "custom lint 1", "command": "lints/one.sh" }
  ],
  "global_fixes": [
    { "name": "custom fix 1", "command": "fixes/one.sh" }
  ]
}
"#;
            let have = Config::parse(give, "test.json").unwrap();
            let want = Config {
                schema: None,
                global_lints: Some(vec![ToolDefinition {
                    name: Some(S("custom lint 1")),
                    command: S("lints/one.sh"),
                }]),
                global_fixes: Some(vec![ToolDefinition {
                    name: Some(S("custom fix 1")),
                    command: S("fixes/one.sh"),
                }]),
                ignore_files: None,
                applications: None,
                stacks: None,
                tests: None,
                commands: None,
            };
            pretty::assert_eq!(have, want);
        }

        #[test]
        fn schema_key_is_allowed() {
            let give = r#"{ "$schema": "./docs/schema.json", "ignore-files": ["a.css"] }"#;
            let have = Config::parse(give, "test.json").unwrap();
            let want = Config {
                schema: Some(S("./docs/schema.json")),
                global_lints: None,
                global_fixes: None,
                ignore_files: Some(vec![S("a.css")]),
                applications: None,
                stacks: None,
                tests: None,
                commands: None,
            };
            pretty::assert_eq!(have, want);
        }

        #[test]
        fn tests() {
            let give = r#"
{
  "tests": [
    { "command": "make unit" },
    { "name": "E2E tests", "command": "make cuke" }
  ]
}
"#;
            let have = Config::parse(give, "test.json").unwrap();
            let want = Config {
                schema: None,
                global_fixes: None,
                global_lints: None,
                ignore_files: None,
                applications: None,
                stacks: None,
                tests: Some(vec![
                    ToolDefinition {
                        name: None,
                        command: S("make unit"),
                    },
                    ToolDefinition {
                        name: Some(S("E2E tests")),
                        command: S("make cuke"),
                    },
                ]),
                commands: None,
            };
            pretty::assert_eq!(have, want);
        }

        #[test]
        fn commands() {
            let give = r#"
{
  "commands": {
    "pitstop": {
      "test": ["unit"],
    }
  }
}
"#;
            let have = Config::parse(give, "test.json").unwrap();
            let want = Config {
                schema: None,
                global_fixes: None,
                global_lints: None,
                ignore_files: None,
                applications: None,
                stacks: None,
                tests: None,
                commands: Some(CommandsSection {
                    pitstop: Some(CommandConfig {
                        test: Some(vec![S("unit")]),
                    }),
                    ..Default::default()
                }),
            };
            pretty::assert_eq!(have, want);
        }

        #[test]
        fn unknown_command() {
            let give = r#"{ "commands": { "zonk": { "test": ["unit"] } } }"#;
            let have = Config::parse(give, "test.json").unwrap_err();
            let UserError::ConfigCannotParse { filename, err } = have else {
                panic!("expected ConfigCannotParse, got {have:?}");
            };
            assert_eq!(filename, "test.json");
            assert!(
                err.contains("unknown field `zonk`"),
                "error should mention the unknown field, got: {err}"
            );
        }

        #[test]
        fn unknown_key() {
            let give = r#"{ "unknown-key": true }"#;
            let have = Config::parse(give, "test.json").unwrap_err();
            let UserError::ConfigCannotParse { filename, err } = have else {
                panic!("expected ConfigCannotParse, got {have:?}");
            };
            assert_eq!(filename, "test.json");
            assert!(
                err.contains("unknown field `unknown-key`"),
                "error should mention the unknown field, got: {err}"
            );
        }
    }

    mod applications {
        use crate::config::{ApplicationNoFile, ApplicationSection, ApplicationWithFile, Config};

        fn disabled_no_file() -> ApplicationNoFile {
            ApplicationNoFile {
                enabled: Some(false),
                operations: None,
            }
        }

        fn disabled_with_file() -> ApplicationWithFile {
            ApplicationWithFile {
                enabled: Some(false),
                ignore_files: None,
                operations: None,
            }
        }

        #[test]
        fn parses_all() {
            let give = r#"
{
  "applications": {
    "actionlint": { "enabled": false },
    "biome": { "enabled": false },
    "checkstyle": { "enabled": false },
    "delete-empty-folders": { "enabled": false },
    "dprint": { "enabled": false },
    "gherkin-lint": { "enabled": false },
    "ghokin": { "enabled": false },
    "git-diff-check": { "enabled": false },
    "gofumpt": { "enabled": false },
    "golangci-lint": { "enabled": false },
    "hadolint": { "enabled": false },
    "keep-sorted": { "enabled": false },
    "prettier": { "enabled": false },
    "pyright": { "enabled": false },
    "ruff": { "enabled": false },
    "rumdl": { "enabled": false },
    "sqlfmt": { "enabled": false },
    "taplo": { "enabled": false },
    "text-runner": { "enabled": false },
    "tikibase": { "enabled": false }
  }
}
"#;
            let have = Config::parse(give, "test.json").unwrap();
            pretty::assert_eq!(
                have.applications,
                Some(ApplicationSection {
                    actionlint: Some(disabled_no_file()),
                    biome: Some(disabled_with_file()),
                    checkstyle: Some(disabled_no_file()),
                    delete_empty_folders: Some(disabled_no_file()),
                    dprint: Some(disabled_with_file()),
                    gherkin_lint: Some(disabled_with_file()),
                    ghokin: Some(disabled_with_file()),
                    git_diff_check: Some(disabled_no_file()),
                    gofumpt: Some(disabled_with_file()),
                    golangci_lint: Some(disabled_no_file()),
                    hadolint: Some(disabled_with_file()),
                    keep_sorted: Some(disabled_with_file()),
                    prettier: Some(disabled_with_file()),
                    pyright: Some(disabled_with_file()),
                    ruff: Some(disabled_with_file()),
                    rumdl: Some(disabled_with_file()),
                    sqlfmt: Some(disabled_with_file()),
                    taplo: Some(disabled_with_file()),
                    text_runner: Some(disabled_no_file()),
                    tikibase: Some(disabled_no_file()),
                })
            );
        }
    }

    mod keep_sorted {
        use crate::config::{ApplicationSection, ApplicationWithFile, Config};
        use big_s::S;

        #[test]
        fn empty() {
            let have = Config::parse("", "test.json").unwrap();
            assert_eq!(have.applications, None);
            assert_eq!(have.keep_sorted(), None);
        }

        #[test]
        fn both_given() {
            let give = r#"{ "applications": { "keep-sorted": { "enabled": true, "ignore-files": ["README.md"] } } }"#;
            let have = Config::parse(give, "test.json").unwrap();
            assert_eq!(
                have.applications,
                Some(ApplicationSection {
                    keep_sorted: Some(ApplicationWithFile {
                        enabled: Some(true),
                        ignore_files: Some(vec![S("README.md")]),
                        operations: None,
                    }),
                    ..Default::default()
                })
            );
        }

        mod enabled {
            use crate::config::{ApplicationSection, ApplicationWithFile, Config};
            use big_s::S;

            #[test]
            fn enabled() {
                let give = r#"{ "applications": { "keep-sorted": { "enabled": true } } }"#;
                let have = Config::parse(give, "test.json").unwrap();
                assert_eq!(
                    have.applications,
                    Some(ApplicationSection {
                        keep_sorted: Some(ApplicationWithFile {
                            enabled: Some(true),
                            ignore_files: None,
                            operations: None,
                        }),
                        ..Default::default()
                    })
                );
            }

            #[test]
            fn disabled() {
                let give = r#"{ "applications": { "keep-sorted": { "enabled": false } } }"#;
                let have = Config::parse(give, "test.json").unwrap();
                assert_eq!(
                    have.applications,
                    Some(ApplicationSection {
                        keep_sorted: Some(ApplicationWithFile {
                            enabled: Some(false),
                            ignore_files: None,
                            operations: None,
                        }),
                        ..Default::default()
                    })
                );
            }

            #[test]
            fn null() {
                let give = r#"{ "applications": { "keep-sorted": { "enabled": null } } }"#;
                let have = Config::parse(give, "test.json").unwrap();
                assert_eq!(
                    have.applications,
                    Some(ApplicationSection {
                        keep_sorted: Some(ApplicationWithFile {
                            enabled: None,
                            ignore_files: None,
                            operations: None,
                        }),
                        ..Default::default()
                    })
                );
            }

            #[test]
            fn missing() {
                let give =
                    r#"{ "applications": { "keep-sorted": { "ignore-files": ["README.md"] } } }"#;
                let have = Config::parse(give, "test.json").unwrap();
                assert_eq!(
                    have.applications,
                    Some(ApplicationSection {
                        keep_sorted: Some(ApplicationWithFile {
                            enabled: None,
                            ignore_files: Some(vec![S("README.md")]),
                            operations: None,
                        }),
                        ..Default::default()
                    })
                );
            }
        }

        mod ignore_files {
            use crate::config::{ApplicationSection, ApplicationWithFile, Config};
            use big_s::S;

            #[test]
            fn empty() {
                let give = r#"{ "applications": { "keep-sorted": { "ignore-files": [] } } }"#;
                let have = Config::parse(give, "test.json").unwrap();
                pretty::assert_eq!(
                    have.applications,
                    Some(ApplicationSection {
                        keep_sorted: Some(ApplicationWithFile {
                            enabled: None,
                            ignore_files: Some(vec![]),
                            operations: None,
                        }),
                        ..Default::default()
                    })
                );
            }

            #[test]
            fn given() {
                let give =
                    r#"{ "applications": { "keep-sorted": { "ignore-files": ["README.md"] } } }"#;
                let have = Config::parse(give, "test.json").unwrap();
                pretty::assert_eq!(
                    have.applications,
                    Some(ApplicationSection {
                        keep_sorted: Some(ApplicationWithFile {
                            enabled: None,
                            ignore_files: Some(vec![S("README.md")]),
                            operations: None,
                        }),
                        ..Default::default()
                    })
                );
            }

            #[test]
            fn missing() {
                let give = r#"{ "applications": { "keep-sorted": { "enabled": true } } }"#;
                let have = Config::parse(give, "test.json").unwrap();
                pretty::assert_eq!(
                    have.applications,
                    Some(ApplicationSection {
                        keep_sorted: Some(ApplicationWithFile {
                            enabled: Some(true),
                            ignore_files: None,
                            operations: None,
                        }),
                        ..Default::default()
                    })
                );
            }
        }
    }

    mod enabled {
        use crate::config::Application;
        use crate::config::ApplicationWithFileOperation;

        #[test]
        fn none() {
            let give = ApplicationWithFileOperation {
                enabled: None,
                ignore_files: None,
            };
            assert!(give.enabled());
        }

        #[test]
        fn enabled() {
            let give = ApplicationWithFileOperation {
                enabled: Some(true),
                ignore_files: None,
            };
            assert!(give.enabled());
        }

        #[test]
        fn disabled() {
            let give = ApplicationWithFileOperation {
                enabled: Some(false),
                ignore_files: None,
            };
            assert!(!give.enabled());
        }
    }

    mod app_enabled {
        use crate::apps::taplo::Taplo;
        use crate::config::{ApplicationSection, ApplicationWithFile, Config};

        #[test]
        fn missing_applications() {
            let config = Config {
                applications: None,
                ..Default::default()
            };
            let have = config.app_enabled(&Taplo {});
            let want = true;
            assert_eq!(have, want);
        }

        #[test]
        fn missing_app() {
            let config = Config {
                applications: Some(ApplicationSection {
                    taplo: None,
                    ..Default::default()
                }),
                ..Default::default()
            };
            let have = config.app_enabled(&Taplo {});
            let want = true;
            assert_eq!(have, want);
        }

        #[test]
        fn unset() {
            let config = Config {
                applications: Some(ApplicationSection {
                    taplo: Some(ApplicationWithFile {
                        enabled: None,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            };
            let have = config.app_enabled(&Taplo {});
            let want = true;
            assert_eq!(have, want);
        }

        #[test]
        fn enabled() {
            let config = Config {
                applications: Some(ApplicationSection {
                    taplo: Some(ApplicationWithFile {
                        enabled: Some(true),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            };
            let have = config.app_enabled(&Taplo {});
            let want = true;
            assert_eq!(have, want);
        }

        #[test]
        fn disabled() {
            let config = Config {
                applications: Some(ApplicationSection {
                    taplo: Some(ApplicationWithFile {
                        enabled: Some(false),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            };
            let have = config.app_enabled(&Taplo {});
            let want = false;
            assert_eq!(have, want);
        }
    }

    mod operations {
        use crate::config::{
            ApplicationSection, ApplicationWithFile, ApplicationWithFileOperation, Config,
            Operations,
        };
        use crate::domain::UserError;
        use big_s::S;

        #[test]
        fn parse_lint_ignore_files() {
            let give = r#"
{
  "applications": {
    "taplo": {
      "operations": {
        "lint": {
          "ignore-files": ["Cargo.toml"]
        }
      }
    }
  }
}
"#;
            let have = Config::parse(give, "test.json").unwrap();
            pretty::assert_eq!(
                have.applications,
                Some(ApplicationSection {
                    taplo: Some(ApplicationWithFile {
                        enabled: None,
                        ignore_files: None,
                        operations: Some(Operations {
                            lint: Some(ApplicationWithFileOperation {
                                enabled: None,
                                ignore_files: Some(vec![S("Cargo.toml")]),
                            }),
                            ..Default::default()
                        }),
                    }),
                    ..Default::default()
                })
            );
        }

        #[test]
        fn unknown_operation_key() {
            let give = r#"
{
  "applications": {
    "taplo": {
      "operations": {
        "unknown": { "enabled": false }
      }
    }
  }
}
"#;
            let have = Config::parse(give, "test.json").unwrap_err();
            let UserError::ConfigCannotParse { filename, err } = have else {
                panic!("expected ConfigCannotParse, got {have:?}");
            };
            assert_eq!(filename, "test.json");
            assert!(
                err.contains("unknown field `unknown`"),
                "error should mention the unknown field, got: {err}"
            );
        }

        #[test]
        fn ignore_files_rejected_for_no_file_app() {
            let give = r#"
{
  "applications": {
    "actionlint": {
      "operations": {
        "lint": {
          "ignore-files": ["foo.yml"]
        }
      }
    }
  }
}
"#;
            let have = Config::parse(give, "test.json").unwrap_err();
            let UserError::ConfigCannotParse { filename, err } = have else {
                panic!("expected ConfigCannotParse, got {have:?}");
            };
            assert_eq!(filename, "test.json");
            assert!(
                err.contains("unknown field `ignore-files`"),
                "error should mention the unknown field, got: {err}"
            );
        }
    }

    mod operation_enabled {
        use crate::apps::taplo::Taplo;
        use crate::config::{
            ApplicationSection, ApplicationWithFile, ApplicationWithFileOperation, Config,
            Operation, Operations,
        };

        #[test]
        fn missing_config_defaults_to_enabled() {
            let config = Config::default();
            assert!(config.operation_enabled(&Taplo {}, Operation::Lint));
            assert!(config.operation_enabled(&Taplo {}, Operation::Fix));
        }

        #[test]
        fn app_disabled_disables_all_operations() {
            let config = Config {
                applications: Some(ApplicationSection {
                    taplo: Some(ApplicationWithFile {
                        enabled: Some(false),
                        operations: Some(Operations {
                            lint: Some(ApplicationWithFileOperation {
                                enabled: Some(true),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            };
            assert!(!config.operation_enabled(&Taplo {}, Operation::Lint));
            assert!(!config.operation_enabled(&Taplo {}, Operation::Fix));
        }

        #[test]
        fn operation_disabled() {
            let config = Config {
                applications: Some(ApplicationSection {
                    taplo: Some(ApplicationWithFile {
                        operations: Some(Operations {
                            lint: Some(ApplicationWithFileOperation {
                                enabled: Some(false),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            };
            assert!(!config.operation_enabled(&Taplo {}, Operation::Lint));
            assert!(config.operation_enabled(&Taplo {}, Operation::Fix));
        }

        #[test]
        fn both_enabled() {
            let config = Config {
                applications: Some(ApplicationSection {
                    taplo: Some(ApplicationWithFile {
                        enabled: Some(true),
                        operations: Some(Operations {
                            lint: Some(ApplicationWithFileOperation {
                                enabled: Some(true),
                                ..Default::default()
                            }),
                            fix: Some(ApplicationWithFileOperation {
                                enabled: Some(true),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            };
            assert!(config.operation_enabled(&Taplo {}, Operation::Lint));
            assert!(config.operation_enabled(&Taplo {}, Operation::Fix));
        }
    }

    mod tool_definition {
        use crate::config::{Operation, ToolDefinition};
        use crate::domain::StackType;
        use big_s::S;

        #[test]
        fn named_tool_includes_operation_and_stack() {
            let tool = ToolDefinition {
                name: Some(S("run-that-app-sorted")),
                command: S("echo hello"),
            };
            let executable = tool.to_executable(Operation::Fix, StackType::Cucumber);
            pretty::assert_eq!(executable.name, "fix Cucumber (run-that-app-sorted)");
        }

        #[test]
        fn unnamed_tool_falls_back_to_command() {
            let tool = ToolDefinition {
                name: None,
                command: S("echo hello"),
            };
            let executable = tool.to_executable(Operation::Lint, StackType::Python);
            pretty::assert_eq!(executable.name, "lint Python (echo hello)");
        }

        #[test]
        fn unsafe_fix_uses_hyphenated_operation() {
            let tool = ToolDefinition {
                name: Some(S("my fix")),
                command: S("echo hello"),
            };
            let executable = tool.to_executable(Operation::FixUnsafe, StackType::Toml);
            pretty::assert_eq!(executable.name, "unsafe-fix TOML (my fix)");
        }
    }

    mod ignores_for {
        use crate::apps::taplo::Taplo;
        use crate::config::{
            ApplicationSection, ApplicationWithFile, ApplicationWithFileOperation, Config,
            Operation, Operations,
        };
        use big_s::S;
        use std::path::Path;

        #[test]
        fn unions_app_and_operation_patterns() {
            let config = Config {
                applications: Some(ApplicationSection {
                    taplo: Some(ApplicationWithFile {
                        ignore_files: Some(vec![S("app.toml")]),
                        operations: Some(Operations {
                            lint: Some(ApplicationWithFileOperation {
                                enabled: None,
                                ignore_files: Some(vec![S("lint.toml")]),
                            }),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            };
            let lint = config.ignores_for(&Taplo {}, Operation::Lint).unwrap();
            assert!(lint.matches_self(Path::new("app.toml"), false));
            assert!(lint.matches_self(Path::new("lint.toml"), false));
            assert!(!lint.matches_self(Path::new("other.toml"), false));
            let fix = config.ignores_for(&Taplo {}, Operation::Fix).unwrap();
            assert!(fix.matches_self(Path::new("app.toml"), false));
            assert!(!fix.matches_self(Path::new("lint.toml"), false));
            assert!(!fix.matches_self(Path::new("other.toml"), false));
        }
    }

    mod select_tests {
        use crate::config::{Config, ToolDefinition};
        use crate::domain::UserError;
        use big_s::S;

        #[test]
        fn no_names_given() {
            let unit_test = ToolDefinition {
                name: Some(S("unit")),
                command: S("echo unit"),
            };
            let cuke_test = ToolDefinition {
                name: Some(S("cuke")),
                command: S("echo cuke"),
            };
            let config = Config {
                tests: Some(vec![unit_test.clone(), cuke_test.clone()]),
                ..Default::default()
            };
            let have = config.select_tests(&[]).unwrap();
            let want = vec![&unit_test, &cuke_test];
            pretty::assert_eq!(have, want);
        }

        #[test]
        fn names_given() {
            let unit_test = ToolDefinition {
                name: Some(S("unit")),
                command: S("echo unit"),
            };
            let cuke_test = ToolDefinition {
                name: Some(S("cuke")),
                command: S("echo cuke"),
            };
            let slow_test = ToolDefinition {
                name: Some(S("slow")),
                command: S("echo slow"),
            };
            let config = Config {
                tests: Some(vec![unit_test.clone(), cuke_test.clone(), slow_test]),
                ..Default::default()
            };
            let have = config.select_tests(&[S("cuke"), S("unit")]).unwrap();
            let want = vec![&cuke_test, &unit_test];
            pretty::assert_eq!(have, want);
        }

        #[test]
        fn duplicate_names_given() {
            let unit_test = ToolDefinition {
                name: Some(S("unit")),
                command: S("echo unit"),
            };
            let config = Config {
                tests: Some(vec![unit_test.clone()]),
                ..Default::default()
            };
            let have = config.select_tests(&[S("unit"), S("unit")]).unwrap();
            let want = vec![&unit_test];
            pretty::assert_eq!(have, want);
        }

        #[test]
        fn unnamed_tests() {
            let unit_test = ToolDefinition {
                name: None,
                command: S("actionlint"),
            };
            let config = Config {
                tests: Some(vec![unit_test.clone()]),
                ..Default::default()
            };
            let have = config.select_tests(&[S("actionlint")]).unwrap();
            let want = vec![&unit_test];
            pretty::assert_eq!(have, want);
        }

        #[test]
        fn unknown_name() {
            let config = Config {
                tests: Some(vec![
                    ToolDefinition {
                        name: Some(S("unit")),
                        command: S("echo unit"),
                    },
                    ToolDefinition {
                        name: Some(S("cuke")),
                        command: S("echo cuke"),
                    },
                ]),
                ..Default::default()
            };
            let have = config.select_tests(&[S("unit"), S("missing")]).unwrap_err();
            let want = UserError::UnknownTest {
                names: vec![S("missing")],
                available: vec![S("unit"), S("cuke")],
            };
            pretty::assert_eq!(have, want);
        }

        #[test]
        fn unknown_name_with_no_configured_tests() {
            let config = Config {
                tests: None,
                ..Default::default()
            };
            let have = config.select_tests(&[S("unit")]).unwrap_err();
            let want = UserError::UnknownTest {
                names: vec![S("unit")],
                available: vec![],
            };
            pretty::assert_eq!(have, want);
        }
    }

    mod tests_for {
        use crate::config::{CommandConfig, CommandsSection, Config, DefaultTests, ToolDefinition};
        use big_s::S;

        #[test]
        fn cli_overrides_config() {
            let config = Config {
                tests: Some(vec![
                    ToolDefinition {
                        name: Some(S("cuke")),
                        command: S("echo cuke"),
                    },
                    ToolDefinition {
                        name: Some(S("unit")),
                        command: S("echo unit"),
                    },
                ]),
                commands: Some(CommandsSection {
                    pitstop: Some(CommandConfig {
                        test: Some(vec![S("unit")]),
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            };
            let cli = &[S("cuke")];
            let have = config
                .tests_for(
                    cli,
                    |commands| commands.pitstop.as_ref(),
                    DefaultTests::None,
                )
                .unwrap();
            pretty::assert_eq!(
                have,
                vec![&ToolDefinition {
                    name: Some(S("cuke")),
                    command: S("echo cuke"),
                }]
            );
        }

        #[test]
        fn empty_cli_uses_config() {
            let config = Config {
                tests: Some(vec![
                    ToolDefinition {
                        name: Some(S("cuke")),
                        command: S("echo cuke"),
                    },
                    ToolDefinition {
                        name: Some(S("unit")),
                        command: S("echo unit"),
                    },
                ]),
                commands: Some(CommandsSection {
                    pitstop: Some(CommandConfig {
                        test: Some(vec![S("unit")]),
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            };
            let cli = &[];
            let have = config
                .tests_for(
                    cli,
                    |commands| commands.pitstop.as_ref(),
                    DefaultTests::None,
                )
                .unwrap();
            pretty::assert_eq!(
                have,
                vec![&ToolDefinition {
                    name: Some(S("unit")),
                    command: S("echo unit"),
                }]
            );
        }

        #[test]
        fn missing_config_uses_default_all() {
            let config = Config {
                tests: Some(vec![
                    ToolDefinition {
                        name: Some(S("cuke")),
                        command: S("echo cuke"),
                    },
                    ToolDefinition {
                        name: Some(S("unit")),
                        command: S("echo unit"),
                    },
                ]),
                commands: None,
                ..Default::default()
            };
            let cli = &[];
            let have = config
                .tests_for(cli, |commands| commands.pitstop.as_ref(), DefaultTests::All)
                .unwrap();
            pretty::assert_eq!(
                have,
                vec![
                    &ToolDefinition {
                        name: Some(S("cuke")),
                        command: S("echo cuke"),
                    },
                    &ToolDefinition {
                        name: Some(S("unit")),
                        command: S("echo unit"),
                    },
                ]
            );
        }

        #[test]
        fn missing_config_uses_default_none() {
            let config = Config {
                tests: Some(vec![
                    ToolDefinition {
                        name: Some(S("cuke")),
                        command: S("echo cuke"),
                    },
                    ToolDefinition {
                        name: Some(S("unit")),
                        command: S("echo unit"),
                    },
                ]),
                commands: None,
                ..Default::default()
            };
            let cli = &[];
            let have = config
                .tests_for(
                    cli,
                    |commands| commands.pitstop.as_ref(),
                    DefaultTests::None,
                )
                .unwrap();
            pretty::assert_eq!(have, Vec::<&ToolDefinition>::new());
        }

        #[test]
        fn empty_command_tests_configured() {
            let config = Config {
                tests: Some(vec![
                    ToolDefinition {
                        name: Some(S("unit")),
                        command: S("echo unit"),
                    },
                    ToolDefinition {
                        name: Some(S("cuke")),
                        command: S("echo cuke"),
                    },
                ]),
                commands: Some(CommandsSection {
                    pitstop: Some(CommandConfig { test: Some(vec![]) }),
                    ..Default::default()
                }),
                ..Default::default()
            };
            let have = config
                .tests_for(&[], |commands| commands.pitstop.as_ref(), DefaultTests::All)
                .unwrap();
            assert_eq!(have, Vec::<&ToolDefinition>::new());
        }
    }
}
