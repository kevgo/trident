use crate::compare_lines_any_order::CompareResult;
use regex::Regex;

/// Verifies that each line in `have` matches exactly one regex in `want`, in any order.
/// Each regex must match the entire line.
pub fn compare_regexes_any_order<'a>(have: &'a [&str], want: &'a [&str]) -> CompareResult<'a> {
    let regexes: Vec<Regex> = want
        .iter()
        .map(|pattern| {
            Regex::new(&format!("^(?:{pattern})$"))
                .unwrap_or_else(|err| panic!("invalid regex '{pattern}': {err}"))
        })
        .collect();
    // for each regex, the indexes of the lines in have that it matches
    let candidates: Vec<Vec<usize>> = regexes
        .iter()
        .map(|regex| {
            have.iter()
                .enumerate()
                .filter(|(_, line)| regex.is_match(line))
                .map(|(i, _)| i)
                .collect()
        })
        .collect();
    // for each line in have, the index of the regex that it is assigned to
    let mut assignments: Vec<Option<usize>> = vec![None; have.len()];
    let mut missing = Vec::new();
    for (p, pattern) in want.iter().enumerate() {
        let mut visited = vec![false; have.len()];
        if !assign(p, &candidates, &mut assignments, &mut visited) {
            missing.push(*pattern);
        }
    }
    let extra = have
        .iter()
        .zip(&assignments)
        .filter(|(_, assignment)| assignment.is_none())
        .map(|(line, _)| *line)
        .collect();
    CompareResult { missing, extra }
}

/// Tries to assign the given regex to a line, reassigning previously assigned regexes if needed.
/// This is the augmenting path step of Kuhn's bipartite matching algorithm.
/// It ensures that a broad regex doesn't take a line that a more specific regex needs.
fn assign(
    regex: usize,
    candidates: &[Vec<usize>],
    assignments: &mut [Option<usize>],
    visited: &mut [bool],
) -> bool {
    for &line in &candidates[regex] {
        if visited[line] {
            continue;
        }
        visited[line] = true;
        let available = match assignments[line] {
            None => true,
            Some(other_regex) => assign(other_regex, candidates, assignments, visited),
        };
        if available {
            assignments[line] = Some(regex);
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_order() {
        let have = vec!["one 1", "two 2", "three 3"];
        let want = vec![r"one \d", r"two \d", r"three \d"];
        assert!(compare_regexes_any_order(&have, &want).success());
    }

    #[test]
    fn different_order() {
        let have = vec!["three 3", "two 2", "one 1"];
        let want = vec![r"one \d", r"two \d", r"three \d"];
        assert!(compare_regexes_any_order(&have, &want).success());
    }

    #[test]
    fn missing_line() {
        let have = vec!["two 2", "three 3"];
        let want = vec![r"one \d", r"two \d", r"three \d"];
        let result = compare_regexes_any_order(&have, &want);
        assert!(!result.success());
        assert_eq!(result.missing, vec![r"one \d"]);
        assert!(result.extra.is_empty());
    }

    #[test]
    fn extra_line() {
        let have = vec!["one 1", "two 2", "three 3"];
        let want = vec![r"two \d", r"three \d"];
        let result = compare_regexes_any_order(&have, &want);
        assert!(!result.success());
        assert!(result.missing.is_empty());
        assert_eq!(result.extra, vec!["one 1"]);
    }

    #[test]
    fn same_number_different_content() {
        let have = vec!["one 1", "two 2", "two 2"];
        let want = vec![r"one \d", r"one \d", r"two \d"];
        let result = compare_regexes_any_order(&have, &want);
        assert!(!result.success());
        assert_eq!(result.missing, vec![r"one \d"]);
        assert_eq!(result.extra, vec!["two 2"]);
    }

    #[test]
    fn different_content() {
        let have = vec!["one"];
        let want = vec!["two"];
        let result = compare_regexes_any_order(&have, &want);
        assert!(!result.success());
        assert_eq!(result.missing, vec!["two"]);
        assert_eq!(result.extra, vec!["one"]);
    }

    #[test]
    fn broad_regex_does_not_steal_specific_line() {
        let have = vec!["one 1", "two 2"];
        let want = vec![r".*", r"one \d"];
        assert!(compare_regexes_any_order(&have, &want).success());
    }
}
