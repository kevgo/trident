use super::{discover_stacks, resolve_scope};
use crate::apps::git_diff_check;
use crate::apps::git_diff_check::GitDiffCheck;
use crate::cli::input::{RunArgsWithScope, ShowExt};
use crate::cli::output::print_metadata;
use crate::config::{Config, ToolDefinition};
use crate::domain::{DetectedStacks, Operation, Result, StackType};
use crate::subshell::ToolCall;
use crate::{git, subshell};
use ahash::AHashMap;
use nonempty::NonEmpty;
use std::process::ExitCode;

pub fn lint(args: &RunArgsWithScope) -> Result<ExitCode> {
    // step 1: load the config
    let config = Config::load()?;
    let ignores = config.ignores()?;
    let show = args.run.show.unwrap_or(conc::Show::Names);
    let error_on_output = false;
    let sequential = args.run.sequential;
    let stderr_to_stdout = true;
    let repo = git::Repo::load();

    // step 2: discover the stacks
    let scope = resolve_scope(&args.scope, repo.as_ref());
    let stacks = discover_stacks(scope, repo.as_ref(), &ignores)?;
    if show.display_metadata() {
        print_metadata(&stacks);
    }

    // step 3: discover the lints to run
    let lints = determine_lints(&config, &stacks, repo.as_ref())?;
    if show.display_metadata() {
        eprintln!("running {} tools", lints.len());
    }

    // step 4: run the lints
    if lints.is_empty() {
        return Ok(ExitCode::SUCCESS);
    }
    let exit_code = subshell::run(subshell::RunArgs {
        sequences: lints.into_sequences(),
        error_on_output,
        sequential,
        show,
        stderr_to_stdout,
    });
    Ok(exit_code)
}

pub fn determine_lints(
    config: &Config,
    detected_stacks: &DetectedStacks,
    git_repo: Option<&git::Repo>,
) -> Result<Lints> {
    // determine the lints for the stacks
    let mut stack_specific: AHashMap<StackType, Vec<ToolCall>> = AHashMap::new();
    for detected_stack in detected_stacks {
        let stack_type = detected_stack.stack.stack_type();
        let stack_config = config.stack_config(stack_type);
        let stack_executables = stack_specific.entry(stack_type).or_default();
        // schedule either the override lints or the default lints
        let stack_lints = stack_config.and_then(|sc| sc.lint.as_ref());
        if let Some(overrides) = stack_lints.and_then(|lint| lint.replace.as_ref()) {
            stack_executables.extend(
                overrides
                    .iter()
                    .map(|tool| tool.to_call(Operation::Lint, Some(stack_type))),
            );
        } else {
            for default_lint in detected_stack.stack.lints() {
                if config.operation_enabled(default_lint.as_ref(), Operation::Lint)
                    && default_lint.enabled_when().enabled_on_disk()
                    && let executables = default_lint.lint_commands(detected_stack, config)?
                {
                    stack_executables.extend(executables);
                }
            }
        }
        if let Some(additions) = stack_lints.and_then(|lint| lint.add.as_ref()) {
            stack_executables.extend(
                additions
                    .iter()
                    .map(|tool_def| tool_def.to_call(Operation::Lint, Some(stack_type))),
            );
        }
    }

    // determine the runnables for the custom lints
    let mut global: Vec<NonEmpty<ToolCall>> = vec![];
    if let Some(custom_lints) = &config.global_lints {
        for ToolDefinition { name, command } in custom_lints {
            global.push(NonEmpty::new(ToolCall {
                operation: Operation::Lint,
                stack: None,
                name: name.clone().unwrap_or_else(|| command.clone()),
                command: conc::shell_command(command),
            }));
        }
    }

    // determine the Git lint
    if config.operation_enabled(&GitDiffCheck {}, Operation::Lint)
        && let Some(repo) = git_repo
    {
        let executable = git_diff_check::lint_command(repo);
        global.push(NonEmpty::new(executable));
    }

    Ok(Lints {
        global,
        stack_specific,
    })
}

pub struct Lints {
    pub global: Vec<NonEmpty<ToolCall>>,
    pub stack_specific: AHashMap<StackType, Vec<ToolCall>>,
}

impl Lints {
    pub fn len(&self) -> usize {
        let Lints {
            global,
            stack_specific,
        } = self;
        let global_len = global.iter().fold(0, |acc, sequence| acc + sequence.len());
        let stack_specific_len = stack_specific
            .values()
            .fold(0, |acc, executables| acc + executables.len());
        global_len + stack_specific_len
    }

    pub fn is_empty(&self) -> bool {
        let Lints {
            global,
            stack_specific,
        } = self;
        global.is_empty() && stack_specific.values().all(Vec::is_empty)
    }

    pub fn into_sequences(self) -> Vec<NonEmpty<ToolCall>> {
        let Lints {
            global,
            stack_specific,
        } = self;
        let mut result = global;
        result.extend(stack_specific.into_values().flatten().map(NonEmpty::new));
        result
    }
}
