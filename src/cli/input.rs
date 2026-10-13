use crate::domain::{Result, UserError};
use clap::builder::{PossibleValue, PossibleValuesParser, TypedValueParser};
use clap::error::ErrorKind;
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(about = env!("CARGO_PKG_DESCRIPTION"))]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(subcommand_required = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Runs all fixes, lints, and tests on CI
    Ci(RunArgsWithTestAndScope),

    /// Embed into claude-compatible coding agents
    #[command(name = "init:claude")]
    InitClaude(InitArgs),

    /// Create the Trident configuration file
    #[command(name = "init:config")]
    InitConfig(InitArgs),

    /// Install the Git pre-commit hook
    #[command(name = "init:githook")]
    InitGithook(InitArgs),

    /// Apply safe code quality fixes
    Fix(RunArgsWithScope),

    /// Apply advanced fixes that might change behavior
    FixUnsafe(RunArgsWithScope),

    /// Run all lints, fixes, and tests on all files
    Full(RunArgsWithTest),

    /// Find code quality issues
    #[command(visible_alias = "postgenerate")]
    Lint(RunArgsWithScope),

    /// Fix and lint the current work
    Pitstop(RunArgsWithTestAndScope),

    /// Lint uncommitted changes
    Postedit(RunArgs),

    /// Fix staged files before committing, never fails
    Precommit(RunArgs),

    /// Run all tests in parallel
    Test(RunArgsWithTest),

    /// Update third-party tools
    #[command(name = "update:tools")]
    UpdateTools,
}

/// which files a command should process
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum Scope {
    /// uncommitted files
    Uncommitted,
    /// files changed on the current branch
    Branch,
    /// all files in the current directory
    All,
}

/// the `--scope` flag
#[derive(clap::Args)]
pub struct ScopeArg {
    /// files to apply the operation to
    #[arg(long, ignore_case = true)]
    pub scope: Option<Scope>,
}

impl ScopeArg {
    /// provides the given default when `--scope` was omitted
    #[must_use]
    pub fn unwrap_or(&self, default: Scope) -> Scope {
        self.scope.unwrap_or(default)
    }
}

// `RunArgs` with a `--test` flag
#[derive(clap::Args)]
pub struct RunArgsWithTest {
    #[command(flatten)]
    pub run: RunArgs,

    /// names of tests to run, joined with +
    #[arg(long, value_delimiter = '+', value_name = "NAME")]
    pub test: Vec<String>,
}

// `RunArgs` with a `--scope` flag
#[derive(clap::Args)]
pub struct RunArgsWithScope {
    #[command(flatten)]
    pub run: RunArgs,

    #[command(flatten)]
    pub scope: ScopeArg,
}

// `RunArgs` with `--test` and `--scope` flags
#[derive(clap::Args)]
pub struct RunArgsWithTestAndScope {
    #[command(flatten)]
    pub run: RunArgs,

    #[command(flatten)]
    pub scope: ScopeArg,

    /// names of tests to run, joined with +
    #[arg(long, value_delimiter = '+', value_name = "NAME")]
    pub test: Vec<String>,
}

#[derive(clap::Args)]
pub struct RunArgs {
    /// run all commands one after another instead of concurrently
    #[arg(long)]
    pub sequential: bool,

    /// how much output to display
    #[arg(long, ignore_case = true, value_parser = show_parser())]
    pub show: Option<conc::Show>,
}

impl RunArgs {
    /// provides a `RunArgs` with the show set to the given default if not provided
    #[must_use]
    pub fn with_default_show(self, default_show: conc::Show) -> Self {
        Self {
            show: Some(self.show.unwrap_or(default_show)),
            ..self
        }
    }
}

/// CLI helpers for [`conc::Show`].
pub trait ShowExt {
    /// indicates whether to display metadata about the detected stacks and commands being run
    #[must_use]
    fn display_metadata(self) -> bool;
}

impl ShowExt for conc::Show {
    fn display_metadata(self) -> bool {
        match self {
            Self::Output | Self::Verbose => true,
            Self::Failed | Self::Names => false,
        }
    }
}

/// clap cannot derive `ValueEnum` for `conc::Show` (foreign type), so we parse it manually
fn show_parser() -> impl TypedValueParser<Value = conc::Show> {
    PossibleValuesParser::new([
        PossibleValue::new("failed").help("only output of failed commands"),
        PossibleValue::new("names").help("command names and output of failed commands"),
        PossibleValue::new("output").help("command names and output of all commands"),
        PossibleValue::new("verbose").help("command lines and output of all commands"),
    ])
    .map(|s| match s.to_ascii_lowercase().as_str() {
        "failed" => conc::Show::Failed,
        "names" => conc::Show::Names,
        "output" => conc::Show::Output,
        "verbose" => conc::Show::Verbose,
        _ => unreachable!("PossibleValuesParser prevents this"),
    })
}

#[derive(clap::Args)]
pub struct InitArgs {
    /// Overwrite existing files
    #[arg(long, short, default_value = "false")]
    pub force: bool,
}

pub fn parse() -> Result<Option<Command>> {
    match Cli::try_parse() {
        Ok(cli) => Ok(cli.command),
        Err(err) => match err.kind() {
            ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
                let _ = err.print();
                Ok(None)
            }
            _ => Err(UserError::Cli {
                msg: err.to_string(),
            }),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Cli, Command, RunArgs, RunArgsWithScope, RunArgsWithTest, RunArgsWithTestAndScope, Scope,
    };
    use clap::Parser;
    use clap::error::ErrorKind;
    use maplit::hashmap;

    fn parse_ci(args: &[&str]) -> RunArgsWithTestAndScope {
        match parse_command("ci", args) {
            Command::Ci(ci) => ci,
            _ => panic!("expected the ci command"),
        }
    }

    fn parse_fix(args: &[&str]) -> RunArgsWithScope {
        match parse_command("fix", args) {
            Command::Fix(fix) => fix,
            _ => panic!("expected the fix command"),
        }
    }

    fn parse_fix_unsafe(args: &[&str]) -> RunArgsWithScope {
        match parse_command("fix-unsafe", args) {
            Command::FixUnsafe(fix_unsafe) => fix_unsafe,
            _ => panic!("expected the fix-unsafe command"),
        }
    }

    fn parse_lint(args: &[&str]) -> RunArgsWithScope {
        match parse_command("lint", args) {
            Command::Lint(lint) => lint,
            _ => panic!("expected the lint command"),
        }
    }

    fn parse_pitstop(args: &[&str]) -> RunArgsWithTestAndScope {
        match parse_command("pitstop", args) {
            Command::Pitstop(pitstop) => pitstop,
            _ => panic!("expected the pitstop command"),
        }
    }

    fn parse_postedit(args: &[&str]) -> RunArgs {
        match parse_command("postedit", args) {
            Command::Postedit(postedit) => postedit,
            _ => panic!("expected the postedit command"),
        }
    }

    fn parse_precommit(args: &[&str]) -> RunArgs {
        match parse_command("precommit", args) {
            Command::Precommit(precommit) => precommit,
            _ => panic!("expected the precommit command"),
        }
    }

    fn parse_full(args: &[&str]) -> RunArgsWithTest {
        parse_with_test_flag("full", args, |command| match command {
            Command::Full(full) => full,
            _ => panic!("expected the full command"),
        })
    }

    fn parse_test(args: &[&str]) -> RunArgsWithTest {
        match parse_command("test", args) {
            Command::Test(test) => test,
            _ => panic!("expected the test command"),
        }
    }

    fn parse_with_test_flag(
        command: &str,
        args: &[&str],
        extract: impl FnOnce(Command) -> RunArgsWithTest,
    ) -> RunArgsWithTest {
        extract(parse_command(command, args))
    }

    fn parse_command(command: &str, args: &[&str]) -> Command {
        let mut argv = vec!["trident", command];
        argv.extend(args);
        Cli::try_parse_from(argv).unwrap().command.unwrap()
    }

    fn parse_err(command: &str, args: &[&str]) -> clap::Error {
        let mut argv = vec!["trident", command];
        argv.extend(args);
        match Cli::try_parse_from(argv) {
            Ok(_) => panic!("expected CLI parse to fail"),
            Err(err) => err,
        }
    }

    #[test]
    fn ci_test_flag_splits_on_plus() {
        pretty::assert_eq!(parse_ci(&["--test=unit+cuke"]).test, vec!["unit", "cuke"]);
    }

    #[test]
    fn ci_test_flag_accepts_a_single_name() {
        pretty::assert_eq!(parse_ci(&["--test=unit"]).test, vec!["unit"]);
    }

    #[test]
    fn ci_without_test_flag_runs_all_tests() {
        pretty::assert_eq!(parse_ci(&[]).test, Vec::<String>::new());
    }

    #[test]
    fn full_test_flag_splits_on_plus() {
        pretty::assert_eq!(parse_full(&["--test=unit+cuke"]).test, vec!["unit", "cuke"]);
    }

    #[test]
    fn full_test_flag_accepts_a_single_name() {
        pretty::assert_eq!(parse_full(&["--test=unit"]).test, vec!["unit"]);
    }

    #[test]
    fn full_without_test_flag_runs_all_tests() {
        pretty::assert_eq!(parse_full(&[]).test, Vec::<String>::new());
    }

    #[test]
    fn pitstop_test_flag_splits_on_plus() {
        pretty::assert_eq!(
            parse_pitstop(&["--test=unit+cuke"]).test,
            vec!["unit", "cuke"]
        );
    }

    #[test]
    fn pitstop_test_flag_accepts_a_single_name() {
        pretty::assert_eq!(parse_pitstop(&["--test=unit"]).test, vec!["unit"]);
    }

    #[test]
    fn pitstop_without_test_flag_selects_no_tests() {
        pretty::assert_eq!(parse_pitstop(&[]).test, Vec::<String>::new());
    }

    #[test]
    fn test_test_flag_splits_on_plus() {
        pretty::assert_eq!(parse_test(&["--test=unit+cuke"]).test, vec!["unit", "cuke"]);
    }

    #[test]
    fn test_test_flag_accepts_a_single_name() {
        pretty::assert_eq!(parse_test(&["--test=unit"]).test, vec!["unit"]);
    }

    #[test]
    fn test_without_test_flag_runs_all_tests() {
        pretty::assert_eq!(parse_test(&[]).test, Vec::<String>::new());
    }

    #[test]
    fn sequential_flag_given() {
        let args = &["--sequential"];
        assert!(parse_ci(args).run.sequential);
        assert!(parse_fix(args).run.sequential);
        assert!(parse_fix_unsafe(args).run.sequential);
        assert!(parse_full(args).run.sequential);
        assert!(parse_lint(args).run.sequential);
        assert!(parse_pitstop(args).run.sequential);
        assert!(parse_postedit(args).sequential);
        assert!(parse_precommit(args).sequential);
        assert!(parse_test(args).run.sequential);
    }

    #[test]
    fn sequential_flag_omitted() {
        assert!(!parse_ci(&[]).run.sequential);
        assert!(!parse_fix(&[]).run.sequential);
        assert!(!parse_fix_unsafe(&[]).run.sequential);
        assert!(!parse_full(&[]).run.sequential);
        assert!(!parse_lint(&[]).run.sequential);
        assert!(!parse_pitstop(&[]).run.sequential);
        assert!(!parse_postedit(&[]).sequential);
        assert!(!parse_precommit(&[]).sequential);
        assert!(!parse_test(&[]).run.sequential);
    }

    #[test]
    fn sequential_flag_survives_default_show() {
        let args = parse_ci(&["--sequential"])
            .run
            .with_default_show(conc::Show::Output);
        assert!(args.sequential);
        pretty::assert_eq!(args.show, Some(conc::Show::Output));
    }

    #[test]
    fn sequential_flag_rejected_by_init_commands() {
        for command in ["init:claude", "init:config", "init:githook"] {
            let err = parse_err(command, &["--sequential"]);
            pretty::assert_eq!(err.kind(), ErrorKind::UnknownArgument);
        }
    }

    #[test]
    fn scope_flag_values() {
        let tests = hashmap! {
            "uncommitted" => Scope::Uncommitted,
            "branch" => Scope::Branch,
            "all" => Scope::All,
            "BRANCH" => Scope::Branch,
        };
        for (value, want) in tests {
            let flag = format!("--scope={value}");
            pretty::assert_eq!(parse_ci(&[&flag]).scope.scope, Some(want));
            pretty::assert_eq!(parse_fix(&[&flag]).scope.scope, Some(want));
            pretty::assert_eq!(parse_fix_unsafe(&[&flag]).scope.scope, Some(want));
            pretty::assert_eq!(parse_lint(&[&flag]).scope.scope, Some(want));
            pretty::assert_eq!(parse_pitstop(&[&flag]).scope.scope, Some(want));
        }
    }

    #[test]
    fn scope_flag_omitted() {
        pretty::assert_eq!(parse_ci(&[]).scope.scope, None);
        pretty::assert_eq!(parse_fix(&[]).scope.scope, None);
        pretty::assert_eq!(parse_fix_unsafe(&[]).scope.scope, None);
        pretty::assert_eq!(parse_lint(&[]).scope.scope, None);
        pretty::assert_eq!(parse_pitstop(&[]).scope.scope, None);
    }

    #[test]
    fn scope_flag_rejects_unknown_value() {
        let err = parse_err("fix", &["--scope=staged"]);
        pretty::assert_eq!(err.kind(), ErrorKind::InvalidValue);
    }
}
