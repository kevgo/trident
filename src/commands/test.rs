use super::stacks_to_test;
use crate::cli::input::{RunArgsWithTestAndScope, ShowExt};
use crate::config::{Config, DefaultTests};
use crate::domain::Result;
use crate::git::Repo;
use crate::subshell;
use std::process::ExitCode;

pub fn test(args: &RunArgsWithTestAndScope) -> Result<ExitCode> {
    let config = Config::load()?;
    let ignores = config.ignores()?;
    let show = args.run.show.unwrap_or(conc::Show::Names);
    let repo = Repo::load();
    let stacks = stacks_to_test(&config, args.scope.scope, repo.as_ref(), &ignores)?;
    let tests = config.test_sequences(
        &args.test,
        |commands| commands.test.as_ref(),
        DefaultTests::All,
        &stacks,
    )?;
    if show.display_metadata() {
        eprintln!("running {} tasks", tests.len());
    }
    if tests.is_empty() {
        return Ok(ExitCode::SUCCESS);
    }
    let exit_code = subshell::run(subshell::RunArgs {
        sequences: tests,
        error_on_output: false,
        sequential: args.run.sequential,
        show,
        stderr_to_stdout: true,
    });
    Ok(exit_code)
}
