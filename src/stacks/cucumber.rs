use crate::apps::cucumber_sort::CucumberSort;
use crate::apps::gherkin_lint::GherkinLint;
use crate::apps::ghokin::Ghokin;
use crate::domain::{Fix, Lint, Stack, StackType};
use std::path::Path;

pub struct Cucumber;

impl Stack for Cucumber {
    fn stack_type(&self) -> StackType {
        StackType::Cucumber
    }

    fn matches(&self, file: &Path) -> bool {
        file.extension().is_some_and(|ext| ext == "feature")
    }

    fn lints(&self) -> Vec<Box<dyn Lint>> {
        vec![Box::new(GherkinLint {})]
    }

    fn fixes(&self) -> Vec<Box<dyn Fix>> {
        vec![Box::new(CucumberSort), Box::new(Ghokin {})]
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::Stack;
    use crate::stacks::Cucumber;
    use maplit::hashmap;
    use std::path::Path;

    #[test]
    fn has_file() {
        let tests = hashmap! {
            "main.feature" => true,
            "src/features/login.feature" => true,
            "other.txt" => false,
        };
        let cucumber = Cucumber {};
        for (give, want) in tests {
            let have = cucumber.matches(Path::new(give));
            assert_eq!(have, want, "{give:?} -> {have:?}");
        }
    }
}
