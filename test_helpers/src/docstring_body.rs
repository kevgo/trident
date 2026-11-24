/// normalizes the content of a Gherkin docstring
#[must_use]
pub fn docstring_body(content: &str) -> String {
    // remove the leading newline that follows the opening quotes
    let content = content.strip_prefix('\r').unwrap_or(content);
    let content = content.strip_prefix('\n').unwrap_or(content);

    // replace \t with actual tabs
    let content = content.replace("\\t", "\t");

    // add a trailing newline
    // let content = content + "\n";

    // on Windows, replace \n with \r\n
    #[cfg(target_os = "windows")]
    let content = content.replace("\n", "\r\n");

    content
}

#[cfg(test)]
mod tests {
    use super::docstring_body;
    use maplit::hashmap;

    #[test]
    fn empty() {
        let tests = hashmap! {
            "" => "", // empty
            "one\ntwo\n" => "one\ntwo\n", // unchanged content
            "\none\ntwo\n" => "one\ntwo\n", // leading newline
            "\rone\ntwo\n" => "one\ntwo\n", // leading carriage return
            "\r\none\r\ntwo\r\n" => "one\r\ntwo\r\n", // leading CRLF
            "\n\nblank line above\n" => "\nblank line above\n", // leading double newline
            "col1\\tcol2\\tcol3" => "col1\tcol2\tcol3", // escaped tabs
            "\nname\\tversion\n" => "name\tversion\n", // leading newline and escaped tabs
            "line\\nnext" => "line\\nnext", // other escapes
        };
        for (give, want) in tests {
            let have = docstring_body(give);
            pretty::assert_eq!(have, want);
        }
    }
}
