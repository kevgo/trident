use std::fmt::Display;

/// a Trident operation that applications can be configured for
///
/// Operations are more basic primitives than Trident commands.
/// All Trident commands execute variations of these three basic operations on various files.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Operation {
    Lint,
    Fix,
    FixUnsafe,
    Format,
    Test,
}

impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::Lint => f.write_str("lint"),
            Operation::Fix => f.write_str("fix"),
            Operation::FixUnsafe => f.write_str("unsafe-fix"),
            Operation::Format => f.write_str("format"),
            Operation::Test => f.write_str("test"),
        }
    }
}
