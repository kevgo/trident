use super::pitstop::run_tasks;
use crate::cli::input::RunArgsWithTest;
use crate::config::{Config, DefaultTests, to_sequences};
use crate::domain::{Operation, Result};
use crate::git::Repo;
use crate::stacks;
use std::process::ExitCode;

pub fn full(args: &RunArgsWithTest) -> Result<ExitCode> {
    let config = Config::load()?;
    let ignores = config.ignores()?;
    let repo = Repo::load();
    let stacks = stacks::discover_all(&ignores);
    let tests = to_sequences(
        config.tests_for(
            &args.test,
            |commands| commands.full.as_ref(),
            DefaultTests::All,
        )?,
        Operation::Test,
        None,
    );
    run_tasks(&args.run, &config, &stacks, repo.as_ref(), tests)
}
