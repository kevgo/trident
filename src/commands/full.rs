use super::pitstop::run_tasks;
use super::stacks_to_test;
use crate::cli::input::{RunArgsWithTest, Scope};
use crate::config::{Config, DefaultTests};
use crate::domain::Result;
use crate::git::Repo;
use crate::stacks;
use std::process::ExitCode;

pub fn full(args: &RunArgsWithTest) -> Result<ExitCode> {
    let config = Config::load()?;
    let ignores = config.ignores()?;
    let repo = Repo::load();
    let stacks = stacks::discover_all(&ignores);
    let tests = config.test_sequences(
        &args.test,
        |commands| commands.full.as_ref(),
        DefaultTests::All,
        &stacks_to_test(&config, Some(Scope::All), repo.as_ref(), &ignores)?,
    )?;
    run_tasks(&args.run, &config, &stacks, repo.as_ref(), tests)
}
