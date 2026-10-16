use crate::apps::hadolint::Hadolint;
use crate::domain::{Fix, Lint, Stack, StackType};
use std::path::Path;

pub struct Dockerfile;

impl Stack for Dockerfile {
    fn stack_type(&self) -> StackType {
        StackType::Dockerfile
    }

    fn matches(&self, file: &Path) -> bool {
        file.file_name().is_some_and(|name| name == "Dockerfile")
    }

    fn lints(&self) -> Vec<Box<dyn Lint>> {
        vec![Box::new(Hadolint {})]
    }

    fn fixes(&self) -> Vec<Box<dyn Fix>> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::Stack;
    use crate::stacks::Dockerfile;
    use maplit::hashmap;
    use std::path::Path;

    #[test]
    fn has_file() {
        let tests = hashmap! {
            "Dockerfile" => true,
            "src/dir/Dockerfile" => true,
            "dockerfile" => false,
            "Dockerfile.dev" => false,
            "other.txt" => false,
        };
        let dockerfile = Dockerfile {};
        for (give, want) in tests {
            let have = dockerfile.matches(Path::new(give));
            assert_eq!(have, want, "{give:?} -> {have:?}");
        }
    }
}
