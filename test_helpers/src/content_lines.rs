/// Yields trimmed non-empty, non-comment lines from `text`.
/// Lines whose trimmed form starts with `#` are treated as comments.
pub(crate) fn content_lines(text: &str) -> impl Iterator<Item = &str> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
}

#[cfg(test)]
mod tests {
    use super::content_lines;

    #[test]
    fn empty_text() {
        let given = "";
        let want: Vec<&str> = vec![];
        let have: Vec<&str> = content_lines(given).collect();
        pretty::assert_eq!(have, want);
    }

    #[test]
    fn keeps_content_lines() {
        let given = "one 1.0.0\ntwo 2.0.0\n";
        let want: Vec<&str> = vec!["one 1.0.0", "two 2.0.0"];
        let have: Vec<&str> = content_lines(given).collect();
        pretty::assert_eq!(have, want);
    }

    #[test]
    fn skips_blank_lines() {
        let given = "\none\n\n  \ntwo\n";
        let want: Vec<&str> = vec!["one", "two"];
        let have: Vec<&str> = content_lines(given).collect();
        pretty::assert_eq!(have, want);
    }

    #[test]
    fn skips_comments() {
        let given = "# header\none\n# trailing\n";
        let want: Vec<&str> = vec!["one"];
        let have: Vec<&str> = content_lines(given).collect();
        pretty::assert_eq!(have, want);
    }

    #[test]
    fn skips_indented_comments() {
        let given = "  # indented\none\n";
        let want: Vec<&str> = vec!["one"];
        let have: Vec<&str> = content_lines(given).collect();
        pretty::assert_eq!(have, want);
    }

    #[test]
    fn trims_surrounding_whitespace() {
        let given = "  one 1.0.0  \n\ttwo\t\n";
        let want: Vec<&str> = vec!["one 1.0.0", "two"];
        let have: Vec<&str> = content_lines(given).collect();
        pretty::assert_eq!(have, want);
    }

    #[test]
    fn keeps_hash_that_is_not_a_comment() {
        let given = "one # not a comment\n";
        let want: Vec<&str> = vec!["one # not a comment"];
        let have: Vec<&str> = content_lines(given).collect();
        pretty::assert_eq!(have, want);
    }
}
