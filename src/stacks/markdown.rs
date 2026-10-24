use crate::apps::rumdl::Rumdl;
use crate::apps::tikibase::Tikibase;
use crate::domain::{Fix, Lint, Stack, StackType};
use std::path::Path;

pub struct Markdown;

impl Stack for Markdown {
    fn stack_type(&self) -> StackType {
        StackType::Markdown
    }

    fn matches(&self, file: &Path) -> bool {
        file.extension().is_some_and(|ext| ext == "md")
    }

    fn lints(&self) -> Vec<Box<dyn Lint>> {
        vec![Box::new(Rumdl {}), Box::new(Tikibase {})]
    }

    fn fixes(&self) -> Vec<Box<dyn Fix>> {
        vec![Box::new(Tikibase {}), Box::new(Rumdl {})]
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::Stack;
    use crate::stacks::Markdown;
    use maplit::hashmap;
    use std::path::Path;

    #[test]
    fn has_file() {
        let tests = hashmap! {
            "README.md" => true,
            "docs/contributing.md" => true,
            "other.txt" => false,
        };
        let markdown = Markdown {};
        for (give, want) in tests {
            let have = markdown.matches(Path::new(give));
            assert_eq!(have, want, "{give:?} -> {have:?}");
        }
    }
}
