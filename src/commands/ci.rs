use super::discover_stacks;
use super::pitstop::run_tasks;
use crate::cli::input::{RunArgsWithTestAndScope, Scope};
use crate::config::{Config, DefaultTests, to_sequences};
use crate::domain::{Operation, Result, UserError};
use crate::git::Repo;
use std::process::ExitCode;

pub fn ci(args: RunArgsWithTestAndScope) -> Result<ExitCode> {
    let repo = Repo::load();
    let before_diff = repo.as_ref().and_then(|repo| repo.diff().ok());

    let config = Config::load()?;
    let ignores = config.ignores()?;
    let scope = args.scope.unwrap_or(Scope::All);
    let stacks = discover_stacks(scope, repo.as_ref(), &ignores)?;
    let args_show = args.run.with_default_show(conc::Show::Output);
    let tests = config.tests_for(
        &args.test,
        |commands| commands.ci.as_ref(),
        DefaultTests::All,
    )?;
    let test_sequences = to_sequences(tests, Operation::Test, None);
    let exit_code = run_tasks(&args_show, &config, &stacks, repo.as_ref(), test_sequences)?;
    if exit_code != ExitCode::SUCCESS {
        return Ok(exit_code);
    }

    let after_diff = repo.as_ref().and_then(|repo| repo.diff().ok());
    if let Some(before_diff) = before_diff
        && let Some(after_diff) = after_diff
        && before_diff != after_diff
    {
        return Err(UserError::CiUnformatted { diff: after_diff });
    }

    Ok(exit_code)
}
