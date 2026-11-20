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
    let stacks = ScopedStacks::all(&ignores);
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
