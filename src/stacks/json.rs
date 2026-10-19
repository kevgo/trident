use crate::apps::prettier::Prettier;
use crate::domain::{Fix, Lint, Stack, StackType};
use std::path::Path;

pub struct Json;

impl Stack for Json {
    fn stack_type(&self) -> StackType {
        StackType::Json
    }

    fn matches(&self, file: &Path) -> bool {
        file.extension().is_some_and(|ext| ext == "json")
    }

    fn lints(&self) -> Vec<Box<dyn Lint>> {
        vec![]
    }

    fn fixes(&self) -> Vec<Box<dyn Fix>> {
        vec![Box::new(Prettier {})]
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::Stack;
    use crate::stacks::Json;
    use maplit::hashmap;
    use std::path::Path;

    #[test]
    fn has_file() {
        let tests = hashmap! {
            "main.json" => true,
            "src/dir/main.json" => true,
            "other.txt" => false,
        };
        let json = Json {};
        for (give, want) in tests {
            let have = json.matches(Path::new(give));
            assert_eq!(have, want, "{give:?} -> {have:?}");
        }
    }
}
