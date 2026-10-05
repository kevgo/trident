use crate::compare_lines_any_order::CompareResult;
use regex::Regex;

/// Order-independent comparison where each `want` entry is a full-line regular expression.
pub fn compare_regexes_any_order(have: &mut Vec<&str>, want: &mut Vec<&str>) -> CompareResult {
    have.sort();
    want.sort();
    let patterns: Vec<Regex> = want
        .iter()
        .map(|pattern| full_line_regex(pattern))
        .collect();
    let edges = edges_between(have, &patterns);
    let pair_have = maximum_matching(have.len(), &edges);

    let mut matched_want = vec![false; want.len()];
    let mut missing = Vec::new();
    for (have_index, want_index) in pair_have.into_iter().enumerate() {
        match want_index {
            Some(want_index) => matched_want[want_index] = true,
            None => missing.push(have[have_index].to_string()),
        }
    }
    let extra = want
        .iter()
        .enumerate()
        .filter(|(index, _)| !matched_want[*index])
        .map(|(_, pattern)| (*pattern).to_string())
        .collect();

    CompareResult { missing, extra }
}

fn full_line_regex(pattern: &str) -> Regex {
    Regex::new(&format!(r"\A(?:{pattern})\z")).unwrap_or_else(|err| {
        panic!("invalid regular expression '{pattern}': {err}");
    })
}

fn edges_between(have: &[&str], patterns: &[Regex]) -> Vec<Vec<usize>> {
    patterns
        .iter()
        .map(|pattern| {
            have.iter()
                .enumerate()
                .filter(|(_, line)| pattern.is_match(line))
                .map(|(index, _)| index)
                .collect()
        })
        .collect()
}

/// Maximum bipartite matching. Sorted order decides ties: earlier patterns and lines are paired first.
fn maximum_matching(have_len: usize, edges: &[Vec<usize>]) -> Vec<Option<usize>> {
    let mut matcher = Matcher {
        edges,
        pair_have: vec![None; have_len],
        seen: vec![false; have_len],
    };
    matcher.solve();
    matcher.pair_have
}

struct Matcher<'a> {
    edges: &'a [Vec<usize>],
    pair_have: Vec<Option<usize>>,
    seen: Vec<bool>,
}

impl Matcher<'_> {
    fn solve(&mut self) {
        for want_index in 0..self.edges.len() {
            self.seen.fill(false);
            self.try_match(want_index);
        }
    }

    fn try_match(&mut self, want_index: usize) -> bool {
        for edge_index in 0..self.edges[want_index].len() {
            let have_index = self.edges[want_index][edge_index];
            if self.seen[have_index] {
                continue;
            }
            self.seen[have_index] = true;
            let can_assign = match self.pair_have[have_index] {
                None => true,
                Some(other_want) => self.try_match(other_want),
            };
            if can_assign {
                self.pair_have[have_index] = Some(want_index);
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_order() {
        let mut left = vec!["one", "two", "three"];
        let mut right = vec!["one", "two", "three"];
        assert!(compare_lines_any_order(&mut left, &mut right).success());
    }

    #[test]
    fn different_order() {
        let mut left = vec!["one", "two", "three"];
        let mut right = vec!["three", "two", "one"];
        assert!(compare_lines_any_order(&mut left, &mut right).success());
    }

    #[test]
    fn missing_line() {
        let mut left = vec!["one", "two", "three"];
        let mut right = vec!["two", "three"];
        let have = compare_lines_any_order(&mut left, &mut right);
        assert!(!have.success());
        assert_eq!(have.missing, vec!["one"]);
        assert!(have.extra.is_empty());
    }

    #[test]
    fn extra_line() {
        let mut left = vec!["two", "three"];
        let mut right = vec!["one", "two", "three"];
        let have = compare_lines_any_order(&mut left, &mut right);
        assert!(!have.success());
        assert!(have.missing.is_empty());
        assert_eq!(have.extra, vec!["one"]);
    }

    #[test]
    fn same_number_different_content() {
        let mut left = vec!["one", "one", "two"];
        let mut right = vec!["one", "two", "two"];
        let have = compare_lines_any_order(&mut left, &mut right);
        assert!(!have.success());
        assert_eq!(have.missing, vec!["one"]);
        assert_eq!(have.extra, vec!["two"]);
    }
}
