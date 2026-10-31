use nonempty::NonEmpty;

use crate::subshell::ToolCall;

pub fn to_conc(sequence: NonEmpty<ToolCall>) -> NonEmpty<conc::Executable> {
    sequence.map(Into::into)
}
