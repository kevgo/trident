use super::pitstop::run_tasks;
use super::{DiscoverStacksArgs, discover_stacks};
use crate::cli::input::{CLIScope, RunArgsWithTestAndScope};
use crate::config::{Config, DefaultTests, to_sequences};
use crate::domain::{Operation, Result, UserError};
use crate::git::Repo;
use std::process::ExitCode;

pub fn ci(
    RunArgsWithTestAndScope { run, scope, test }: RunArgsWithTestAndScope,
) -> Result<ExitCode> {
    let repo = Repo::load();
    let before_diff = repo.as_ref().and_then(|repo| repo.diff().ok());

    let config = Config::load()?;
    let ignores = config.ignores()?;
    let stacks = discover_stacks(DiscoverStacksArgs {
        scope_flag: scope,
        default_scope: Some(CLIScope::All),
        repo: repo.as_ref(),
        ignores: &ignores,
    });
    let args_show = run.with_default_show(conc::Show::Output);
    let tests = config.tests_for(&test, |commands| commands.ci.as_ref(), DefaultTests::All)?;
    let test_sequences = to_sequences(tests, Operation::Test, None);
    let exit_code = run_tasks(
        &args_show,
        &config,
        &stacks,
        &ignores,
        repo.as_ref(),
        test_sequences,
    )?;
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
