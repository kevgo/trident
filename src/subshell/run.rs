use crate::domain::ToolDefinition;
use crate::subshell::Sequence;
use std::process::ExitCode;

pub struct RunArgs {
    pub sequences: NonEmpty<Sequence>,
    pub error_on_output: bool,
    pub sequential: bool,
    pub show: conc::Show,
    pub stderr_to_stdout: bool,
}

pub fn run(
    RunArgs {
        sequences,
        error_on_output,
        sequential,
        show,
        stderr_to_stdout,
    }: RunArgs,
) -> ExitCode {
    let conc_sequences = sequences
        .into_iter()
        .map(|sequence| sequence.into())
        .collect();
    conc::run(conc::RunArgs {
        sequences: conc_sequences,
        error_on_output,
        sequential,
        show,
        stderr_to_stdout,
    })
}
