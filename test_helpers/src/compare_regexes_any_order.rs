use crate::compare_lines_any_order::CompareResult;
use regex::Regex;

/// Verifies that each line in `have` matches exactly one regex in `want`, in any order.
/// Each regex must match the entire line.
pub fn compare_regexes_any_order<'a>(have: &'a [&str], want: &'a [&str]) -> CompareResult<'a> {
    // Uses bipartite matching: matches each regex to exactly one line and vice versa.
    // A regex without a line is reported as `missing`,
    // a line without a regex is reported as `extra`.

    // wrap each pattern in ^(?:...)$ so that it must match the whole line, not just a part of it
    let regexes: Vec<Regex> = want
        .iter()
        .map(|pattern| {
            Regex::new(&format!("^(?:{pattern})$"))
                .unwrap_or_else(|err| panic!("invalid regex '{pattern}': {err}"))
        })
        .collect();
    // for each regex, the indexes of the lines in `have` that it matches
    // (the edges of the bipartite graph)
    let matching_lines: Vec<Vec<usize>> = regexes
        .iter()
        .map(|regex| {
            have.iter()
                .enumerate()
                .filter(|(_, line)| regex.is_match(line))
                .map(|(i, _)| i)
                .collect()
        })
        .collect();
    // for each line in `have`, the index of the regex that it is assigned to
    let mut matching_regexes: Vec<Option<usize>> = vec![None; have.len()];
    let mut missing = Vec::new();
    // add the regexes one at a time. Each call to assign may move regexes added earlier
    // to different lines, but it never drops a regex that already has a line.
    for (p, pattern) in want.iter().enumerate() {
        // visited prevents a single assign call from looking at the same line twice,
        // which would otherwise cause endless recursion. It is reset for each regex.
        let mut visited = vec![false; have.len()];
        if !assign(p, &matching_lines, &mut matching_regexes, &mut visited) {
            // no line is free for this regex, even after moving the other regexes around
            missing.push(*pattern);
        }
    }
    // lines that no regex has claimed are extra
    let extra = have
        .iter()
        .zip(&matching_regexes)
        .filter(|(_, assignment)| assignment.is_none())
        .map(|(line, _)| *line)
        .collect();
    CompareResult { missing, extra }
}

/// Tries to assign the given regex to a line, reassigning previously assigned regexes if needed.
/// This is the augmenting path step of Kuhn's bipartite matching algorithm.
/// It ensures that a broad regex doesn't take a line that a more specific regex needs.
///
/// Returns true if the regex now has a line. In that case `matching_regexes` holds
/// the new pairing: this regex has a line, and every regex that had a line before still has one,
/// possibly a different one.
/// Returns false if there is no way to make room for the regex.
///
/// How it works, for each line that the regex matches:
/// - If the line is free, take it. Done.
/// - If another regex already holds the line, recursively ask that regex to move to
///   one of its other matching lines. If it can move, possibly by pushing yet another regex
///   further along, the line becomes free and this regex takes it.
/// - Otherwise try the next matching line.
///
/// Using the example from `compare_regexes_any_order`: `.*` holds "one 1" when `one \d`
/// arrives. `one \d` asks `.*` to move, `.*` moves to the free "two 2",
/// and `one \d` takes "one 1".
fn assign(
    regex: usize,
    matching_lines: &[Vec<usize>],
    matching_regexes: &mut [Option<usize>],
    visited: &mut [bool],
) -> bool {
    for &line in &matching_lines[regex] {
        if visited[line] {
            // an earlier step of this search already tried this line, either taking it
            // or asking its regex to move, so skip it
            continue;
        }
        visited[line] = true;
        let available = match matching_regexes[line] {
            // nobody holds this line, we can take it
            None => true,
            // another regex holds this line, see if that regex can move to another line
            Some(other_regex) => assign(other_regex, matching_lines, matching_regexes, visited),
        };
        if available {
            // the line is free now: either it was free all along, or its previous regex
            // has moved to another line in the recursive call above
            matching_regexes[line] = Some(regex);
            return true;
        }
    }
    // all matching lines are held by regexes that can't move
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
