use crate::domain::{Fix, Lint, Stack, StackType};
use std::path::Path;

pub struct Unknown;

impl Stack for Unknown {
    fn stack_type(&self) -> StackType {
        StackType::Unknown
    }

    fn matches(&self, _file: &Path) -> bool {
        true
    }

    fn lints(&self) -> Vec<Box<dyn Lint>> {
        vec![]
    }

    fn fixes(&self) -> Vec<Box<dyn Fix>> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::Stack;
    use crate::stacks::Unknown;
    use maplit::hashmap;
    use std::path::Path;

    #[test]
    fn has_file() {
        let tests = hashmap! {
            "main.toml" => true,
            "src/dir/main.toml" => true,
            "other.txt" => true,
            ".gherkin-lintrc" => true,
        };
        let unknown = Unknown {};
        for (give, want) in tests {
            let have = unknown.matches(Path::new(give));
            assert_eq!(have, want, "{give:?} -> {have:?}");
        }
    }
}
