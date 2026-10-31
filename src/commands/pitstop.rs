use crate::cli::input::{RunArgs, RunArgsWithTestAndScope, ShowExt};
use crate::cli::output::print_metadata;
use crate::commands::lint::Lints;
use crate::commands::{discover_stacks, fix, lint, resolve_scope};
use crate::config::{Config, DefaultTests, to_sequences};
use crate::domain::{DetectedStacks, Operation, Result, Runnables, StackType};
use crate::git::Repo;
use crate::subshell::{self, ToolCall};
use ahash::AHashMap;
use nonempty::NonEmpty;
use std::process::ExitCode;

pub fn pitstop(args: &RunArgsWithTestAndScope) -> Result<ExitCode> {
    let config = Config::load()?;
    let ignores = config.ignores()?;
    let repo = Repo::load();
    let scope = resolve_scope(&args.scope, repo.as_ref());
    let stacks = discover_stacks(scope, repo.as_ref(), &ignores)?;
    let tests = to_sequences(
        config.tests_for(
            &args.test,
            |commands| commands.pitstop.as_ref(),
            DefaultTests::None,
        )?,
        Operation::Test,
        None,
    );
    run_tasks(&args.run, &config, &stacks, repo.as_ref(), tests)
}

/// runs global fixes, then stack-specific fixes, then lints on the given stacks
///
/// `tests` run together with the global lints.
pub(crate) fn run_tasks(
    args: &RunArgs,
    config: &Config,
    stacks: &DetectedStacks,
    repo: Option<&Repo>,
    tests: Vec<NonEmpty<ToolCall>>,
) -> Result<ExitCode> {
    let show = args.show.unwrap_or(conc::Show::Names);
    let error_on_output = false;
    let sequential = args.sequential;
    let stderr_to_stdout = true;

    if show.display_metadata() {
        print_metadata(stacks);
    }

    // step 1: discover the runnables
    let fixes = fix::determine_fixes(config, stacks)?;
    let mut lints = lint::determine_lints(config, stacks, repo)?;
    lints.global.extend(tests);
    let tool_count = fixes.len() + lints.len();
    if show.display_metadata() {
        // TODO: print "running XXX tasks" instead of "running XXX tools"
        // we might run the same tool multiple times
        eprintln!("running {tool_count} tools");
    }
    let Runnables {
        global: global_fixes,
        stack_specific: stack_specific_fixes,
    } = fixes;

    // step 2: run the global fixes by themselves first
    if let Some(global_fixes) = global_fixes {
        let exit_code = subshell::run(subshell::RunArgs {
            sequences: vec![global_fixes],
            error_on_output,
            sequential,
            stderr_to_stdout,
            show,
        });
        if exit_code != ExitCode::SUCCESS {
            return Ok(exit_code);
        }
    }

    // step 3: run concurrent sequences of stack-specific fixes and lints
    let mut stack_executables: AHashMap<StackType, Vec<ToolCall>> = AHashMap::new();
    for (stack_type, stack_specific_fixes) in stack_specific_fixes {
        let entry = stack_executables.entry(stack_type).or_default();
        entry.extend(stack_specific_fixes);
    }
    let Lints {
        global: global_lints,
        stack_specific: stack_specific_lints,
    } = lints;
    for (stack_type, stack_type_lints) in stack_specific_lints {
        let entry = stack_executables.entry(stack_type).or_default();
        entry.extend(stack_type_lints);
    }
    let stack_sequences = stack_executables
        .into_values()
        .filter_map(NonEmpty::from_vec)
        .collect();
    let exit_code = subshell::run(subshell::RunArgs {
        sequences: stack_sequences,
        error_on_output,
        sequential,
        show,
        stderr_to_stdout,
    });
    if exit_code != ExitCode::SUCCESS {
        return Ok(exit_code);
    }

    // step 4: run the global lints
    //
    // We need to run them after the stack-specific fixes because
    // some of them, like the global git-diff-check linter,
    // error on whitespace problems and therefore
    // depend on the formatting having been run.
    let exit_code = subshell::run(subshell::RunArgs {
        sequences: global_lints,
        error_on_output,
        sequential,
        show,
        stderr_to_stdout,
    });
    Ok(exit_code)
}
