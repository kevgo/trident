use crate::apps::{actionlint::Actionlint, prettier::Prettier};
use crate::domain::{Fix, Lint, Stack, StackType};
use std::path::Path;

pub struct Yml;

impl Stack for Yml {
    fn stack_type(&self) -> StackType {
        StackType::Yml
    }

    fn matches(&self, file: &Path) -> bool {
        file.extension()
            .is_some_and(|ext| ext == "yml" || ext == "yaml")
    }

    fn lints(&self) -> Vec<Box<dyn Lint>> {
        vec![Box::new(Actionlint {})]
    }

    fn fixes(&self) -> Vec<Box<dyn Fix>> {
        vec![Box::new(Prettier {})]
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::Stack;
    use crate::stacks::Yml;
    use maplit::hashmap;
    use std::path::Path;

    #[test]
    fn has_file() {
        let tests = hashmap! {
            "main.yml" => true,
            "main.yaml" => true,
            "main.yl" => false,
            "other.text" => false,
             "src/dir/main.yml" => true,
        };
        let yml = Yml {};
        for (give, want) in tests {
            let have = yml.matches(Path::new(give));
            assert_eq!(have, want, "{give:?} -> {have:?}");
        }
    }
}
