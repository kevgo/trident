use std::cmp::Ordering;

pub fn compare_lines_any_order<'a>(
    have: &'a mut Vec<&str>,
    want: &'a mut Vec<&str>,
) -> CompareResult<'a> {
    have.sort();
    want.sort();
    let mut missing = Vec::new();
    let mut extra = Vec::new();
    let mut i = 0;
    let mut j = 0;
    while i < have.len() && j < want.len() {
        match have[i].cmp(want[j]) {
            Ordering::Less => {
                extra.push(have[i]);
                i += 1;
            }
            Ordering::Greater => {
                missing.push(want[j]);
                j += 1;
            }
            Ordering::Equal => {
                i += 1;
                j += 1;
            }
        }
    }
    missing.extend(want[j..].iter());
    extra.extend(have[i..].iter());

    CompareResult { missing, extra }
}

pub struct CompareResult<'a> {
    pub missing: Vec<&'a str>,
    pub extra: Vec<&'a str>,
}

impl<'a> CompareResult<'a> {
    pub fn message(&self) -> String {
        let mut message = String::new();
        message.push_str("\nmissing lines:\n");
        for (i, line) in self.missing.iter().enumerate() {
            message.push_str(&format!("{}. '{}'\n", i + 1, line));
        }
        message.push_str("extra lines:\n");
        for (i, line) in self.extra.iter().enumerate() {
            message.push_str(&format!("{}. '{}'\n", i + 1, line));
        }
        message.push_str("end\n");
        message
    }
    pub fn success(&self) -> bool {
        self.missing.is_empty() && self.extra.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_order() {
        let mut have = vec!["one", "two", "three"];
        let mut want = vec!["one", "two", "three"];
        assert!(compare_lines_any_order(&mut have, &mut want).success());
    }

    #[test]
    fn different_order() {
        let mut want = vec!["one", "two", "three"];
        let mut have = vec!["three", "two", "one"];
        assert!(compare_lines_any_order(&mut have, &mut want).success());
    }

    #[test]
    fn missing_line() {
        let mut want = vec!["one", "two", "three"];
        let mut have = vec!["two", "three"];
        let result = compare_lines_any_order(&mut have, &mut want);
        assert!(!result.success());
        assert_eq!(result.missing, vec!["one"]);
        assert!(result.extra.is_empty());
    }

    #[test]
    fn extra_line() {
        let mut want = vec!["two", "three"];
        let mut have = vec!["one", "two", "three"];
        let result = compare_lines_any_order(&mut have, &mut want);
        assert!(!result.success());
        assert!(result.missing.is_empty());
        assert_eq!(result.extra, vec!["one"]);
    }

    #[test]
    fn same_number_different_content() {
        let mut want = vec!["one", "one", "two"];
        let mut have = vec!["one", "two", "two"];
        let result = compare_lines_any_order(&mut have, &mut want);
        assert!(!result.success());
        assert_eq!(result.missing, vec!["one"]);
        assert_eq!(result.extra, vec!["two"]);
    }
}
