use super::discover_stacks;
use super::pitstop::run_tasks;
use crate::cli::input::{RunArgsWithTest, Scope};
use crate::config::{Config, DefaultTests};
use crate::domain::Result;
use crate::git::Repo;
use std::process::ExitCode;

pub fn full(args: &RunArgsWithTest) -> Result<ExitCode> {
    let config = Config::load()?;
    let ignores = config.ignores()?;
    let repo = Repo::load();
    let stacks = discover_stacks(Scope::All, repo.as_ref(), &ignores)?;
    let tests = config.test_sequences(
        &args.test,
        |commands| commands.full.as_ref(),
        DefaultTests::All,
        &stacks,
    )?;
    run_tasks(&args.run, &config, &stacks, repo.as_ref(), tests)
}
