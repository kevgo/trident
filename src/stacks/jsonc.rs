use crate::apps::prettier::Prettier;
use crate::domain::{Fix, Lint, Stack, StackType};
use std::path::Path;

pub struct JsonC;

impl Stack for JsonC {
    fn stack_type(&self) -> StackType {
        StackType::JsonC
    }

    fn matches(&self, file: &Path) -> bool {
        file.extension().is_some_and(|ext| ext == "jsonc")
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
    use crate::stacks::JsonC;
    use maplit::hashmap;
    use std::path::Path;

    #[test]
    fn has_file() {
        let tests = hashmap! {
            "main.jsonc" => true,
            "src/dir/main.jsonc" => true,
            "src/dir/main.json" => false,
            "other.txt" => false,
        };
        let json = JsonC {};
        for (give, want) in tests {
            let have = json.matches(Path::new(give));
            assert_eq!(have, want, "{give:?} -> {have:?}");
        }
    }
}
