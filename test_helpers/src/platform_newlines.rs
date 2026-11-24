/// Line endings for the current operating system: CRLF on Windows, LF elsewhere.
#[must_use]
pub fn platform_newlines(content: &str) -> String {
    let content = crate::standardize_newlines(content);
    if cfg!(windows) {
        content.replace('\n', "\r\n")
    } else {
        content
    }
}

#[cfg(test)]
mod tests {
    use super::platform_newlines;
    use maplit::hashmap;

    #[test]
    fn tests() {
        #[cfg(windows)]
        let tests = hashmap! {
            "" => "", // empty string
            "one" => "one", // no newlines
            "one\ntwo\n" => "one\r\ntwo\r\n", // unix newlines
            "one\r\ntwo\r\n" => "one\r\ntwo\r\n", // windows newlines stay
            "one\rtwo\r" => "one\r\ntwo\r\n", // mac newlines
            "one\r\ntwo\rthree\nfour" => "one\r\ntwo\r\nthree\r\nfour", // mixed newlines
        };
        #[cfg(not(windows))]
        let tests = hashmap! {
            "" => "", // empty string
            "one" => "one", // no newlines
            "one\ntwo\n" => "one\ntwo\n", // unix newlines stay
            "one\r\ntwo\r\n" => "one\ntwo\n", // windows newlines become LF
            "one\rtwo\r" => "one\ntwo\n", // mac newlines become LF
        };
        for (given, want) in tests {
            let have = platform_newlines(given);
            pretty::assert_eq!(have, want);
        }
    }
}
