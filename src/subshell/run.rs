use crate::domain::ToolCall;
use std::process::ExitCode;

pub struct RunArgs {
    pub calls: Vec<Vec<ToolCall>>,
    pub error_on_output: bool,
    pub sequential: bool,
    pub show: conc::Show,
    pub stderr_to_stdout: bool,
}

pub fn run(
    RunArgs {
        calls,
        error_on_output,
        sequential,
        show,
        stderr_to_stdout,
    }: RunArgs,
) -> ExitCode {
    let sequences = calls
        .into_iter()
        .map(|calls| conc::Sequence::from(calls))
        .collect();
    conc::run(conc::RunArgs {
        sequences,
        error_on_output,
        sequential,
        show,
        stderr_to_stdout,
    })
}
