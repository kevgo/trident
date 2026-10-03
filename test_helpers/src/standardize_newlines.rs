pub fn standardize_newlines(content: &str) -> String {
    content.replace("\r\n", "\n").replace("\r", "\n")
}

#[cfg(test)]
mod tests {
    use super::standardize_newlines;
    use maplit::hashmap;

    #[test]
    fn tests() {
        let tests = hashmap! {
            "" => "", // empty string
            "one" => "one", // no newlines
            "one\ntwo\n" => "one\ntwo\n", // unix newlines
            "one\r\ntwo\r\n" => "one\ntwo\n", // windows newlines
            "one\rtwo\r" => "one\ntwo\n", // mac newlines
            "one\r\ntwo\rthree\nfour" => "one\ntwo\nthree\nfour", // mixed newlines
        };
        for (given, want) in tests {
            let have = standardize_newlines(given);
            pretty::assert_eq!(have, want);
        }
    }
}
