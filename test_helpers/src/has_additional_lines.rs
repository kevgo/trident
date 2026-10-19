use crate::content_lines;
use crate::remove_line_matching;
use crate::remove_lines;

/// Verifies that `new` == `old` + `patterns`
/// where each line in `patterns` matches exactly one line in `new`.
/// Ignores empty lines and `#` comments.
pub fn has_additional_lines(old: &str, new: &str, patterns: &str) -> Result<(), String> {
    let mut new_lines: Vec<&str> = content_lines(new).collect();
    // remove `old` from `new_lines`
    let missing_in_new = remove_lines(&mut new_lines, content_lines(old));
    if !missing_in_new.is_empty() {
        return Err(format!(
            "no longer contains lines:\n{}",
            missing_in_new.join("\n")
        ));
    }
    // all remaining lines must match one of the patterns
    for pattern in content_lines(patterns) {
        remove_line_matching(&mut new_lines, pattern)?;
    }
    if !new_lines.is_empty() {
        return Err(format!(
            "unexpected additional lines:\n{}",
            new_lines.join("\n")
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use big_s::S;

    #[test]
    fn added_matching_lines() {
        let old = "";
        let new = "\
# more info at https://github.com/kevgo/run-that-app

biome 0.21.0
rumdl 0.2.14
";
        let patterns = r"
biome \d+\.\d+\.\d+
rumdl \d+\.\d+\.\d+
";
        assert_eq!(has_additional_lines(old, new, patterns), Ok(()));
    }

    #[test]
    fn existing_lines_one_expected_additional() {
        let old = "\
delete-empty-folders 0.0.2
rumdl 0.2.14
";
        let new = "\
# more info at https://github.com/kevgo/run-that-app

delete-empty-folders 0.0.2
rumdl 0.2.14
taplo 0.10.0
";
        let patterns = r"taplo \d+\.\d+\.\d+";
        assert_eq!(has_additional_lines(old, new, patterns), Ok(()));
    }

    #[test]
    fn ignores_changes_to_comments() {
        let old = "\
# old comment

one 1.0.0
";
        let new = "\
# new comment

one 1.0.0
two 2.0.0
";
        let patterns = r"two \d+\.\d+\.\d+";
        assert_eq!(has_additional_lines(old, new, patterns), Ok(()));
    }

    #[test]
    fn unexpected_removal() {
        let old = "\
one 1.0.0
two 2.0.0
";
        let new = "\
one 1.0.0
";
        let patterns = "three 3.0.0";
        let have = has_additional_lines(old, new, patterns);
        assert_eq!(
            have,
            Err(S("\
no longer contains lines:
two 2.0.0"))
        );
    }

    #[test]
    fn pattern_not_added() {
        let old = "";
        let new = "\
biome 0.21.0
";
        let patterns = r"taplo \d+\.\d+\.\d+";
        let have = has_additional_lines(old, new, patterns);
        assert_eq!(
            have,
            Err(S("\
want exactly one new line matching:
taplo \\d+\\.\\d+\\.\\d+
(matched 0)"))
        );
    }

    #[test]
    fn pattern_matches_two_new_lines() {
        let old = "";
        let new = "\
one 1.0.0
one 2.0.0
";
        let have = has_additional_lines(old, new, r"one \d+\.\d+\.\d+");
        assert_eq!(
            have,
            Err(S("\
want exactly one new line matching:
one \\d+\\.\\d+\\.\\d+
(matched 2)"))
        );
    }

    #[test]
    fn unexpected_additional_line() {
        let old = "";
        let new = "\
biome 0.21.0
rumdl 0.2.14
";
        let have = has_additional_lines(old, new, r"rumdl \d+\.\d+\.\d+");
        assert_eq!(
            have,
            Err(S("\
unexpected additional lines:
biome 0.21.0"))
        );
    }

    #[test]
    fn new_lines_can_be_in_any_order() {
        let old = "";
        let new = "\
biome 0.21.0
rumdl 0.2.14
";
        let patterns = r"
rumdl \d+\.\d+\.\d+
biome \d+\.\d+\.\d+
";
        assert_eq!(has_additional_lines(old, new, patterns), Ok(()));
    }

    #[test]
    fn duplicate_previous_lines_must_all_remain() {
        let old = "\
one
one
";
        let new = "\
one
";
        let have = has_additional_lines(old, new, "");
        assert_eq!(
            have,
            Err(S("\
no longer contains lines:
one"))
        );
    }
}
