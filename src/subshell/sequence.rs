use nonempty::NonEmpty;

use crate::subshell::ToolCall;

pub fn to_conc(sequence: NonEmpty<ToolCall>) -> NonEmpty<conc::Executable> {
    sequence
        .into_iter()
        .map(|tool_call| tool_call.into())
        .collect()
}
