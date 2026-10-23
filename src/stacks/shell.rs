use crate::apps::shellcheck::Shellcheck;
use crate::apps::shfmt::Shfmt;
use crate::domain::{Fix, Lint, Stack, StackType};
use std::path::Path;

pub struct Shell;

impl Stack for Shell {
    fn stack_type(&self) -> StackType {
        StackType::Shell
    }

    fn matches(&self, file: &Path) -> bool {
        file.extension().is_some_and(|ext| ext == "sh")
    }

    fn lints(&self) -> Vec<Box<dyn Lint>> {
        vec![Box::new(Shellcheck)]
    }

    fn fixes(&self) -> Vec<Box<dyn Fix>> {
        // The Rust stack does not specify any fixes by default
        // because there are too many rustfmt options.
        // Please define your own rustfmt calls as custom fixes in trident.json.
        vec![Box::new(Shfmt)]
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::Stack;
    use crate::stacks::Shell;
    use maplit::hashmap;
    use std::path::Path;

    #[test]
    fn has_file() {
        let tests = hashmap! {
            "main.sh" => true,
            "src/dir/main.sh" => true,
            "other.txt" => false,
        };
        let rust = Shell {};
        for (give, want) in tests {
            let have = rust.matches(Path::new(give));
            assert_eq!(have, want, "{give:?} -> {have:?}");
        }
    }
}
