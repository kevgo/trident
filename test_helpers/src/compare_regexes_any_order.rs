use crate::compare_lines_any_order::CompareResult;
use regex::Regex;

/// Order-independent comparison where each `want` entry is a full-line regular expression.
pub fn compare_regexes_any_order(have: &mut Vec<&str>, want: &mut Vec<&str>) -> CompareResult {
    have.sort();
    want.sort();
    let patterns: Vec<Regex> = want.iter().copied().map(full_line_regex).collect();
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
    use crate::compare_lines_any_order::compare_lines_any_order;

    fn assert_same_as_literal_compare(have: &[&str], want: &[&str]) {
        let mut have_lines = have.to_vec();
        let mut want_lines = want.to_vec();
        let mut have_regex = have.to_vec();
        let mut want_regex = want.to_vec();
        let lines = compare_lines_any_order(&mut have_lines, &mut want_lines);
        let regexes = compare_regexes_any_order(&mut have_regex, &mut want_regex);
        assert_eq!(lines.missing, regexes.missing);
        assert_eq!(lines.extra, regexes.extra);
    }

    #[test]
    fn literals_match_compare_lines_any_order() {
        let cases: &[(&[&str], &[&str])] = &[
            (&[], &[]),
            (&["one", "two", "three"], &["one", "two", "three"]),
            (&["one", "two", "three"], &["three", "two", "one"]),
            (&["one", "two", "three"], &["two", "three"]),
            (&["two", "three"], &["one", "two", "three"]),
            (&["one", "one", "two"], &["one", "two", "two"]),
            (&["a", "a", "c", "d"], &["a", "b", "b", "d"]),
            (&["only"], &[]),
            (&[], &["only"]),
        ];
        for (have, want) in cases {
            assert_same_as_literal_compare(have, want);
        }
    }

    #[test]
    fn regex_matches_in_any_order() {
        let mut have = vec!["alpha 1", "beta 2"];
        let mut want = vec![r"beta \d+", r"alpha \d+"];
        assert!(compare_regexes_any_order(&mut have, &mut want).success());
    }

    #[test]
    fn pattern_matches_a_whole_line_only() {
        let mut have = vec!["prefix one suffix"];
        let mut want = vec!["one"];
        let result = compare_regexes_any_order(&mut have, &mut want);
        assert_eq!(result.missing, vec!["prefix one suffix"]);
        assert_eq!(result.extra, vec!["one"]);
    }

    #[test]
    fn unanchored_content_can_be_expressed_in_the_pattern() {
        let mut have = vec!["prefix one suffix"];
        let mut want = vec![".*one.*"];
        assert!(compare_regexes_any_order(&mut have, &mut want).success());
    }

    #[test]
    fn alternation_is_anchored_to_the_whole_line() {
        let mut have = vec!["ab"];
        let mut want = vec!["a|b"];
        let result = compare_regexes_any_order(&mut have, &mut want);
        assert_eq!(result.missing, vec!["ab"]);
        assert_eq!(result.extra, vec!["a|b"]);
    }

    #[test]
    fn dot_matches_any_character() {
        let mut have = vec!["abc"];
        let mut want = vec!["a.c"];
        assert!(compare_regexes_any_order(&mut have, &mut want).success());
    }

    #[test]
    fn escaped_dot_is_literal() {
        let mut have = vec!["abc"];
        let mut want = vec![r"a\.c"];
        let result = compare_regexes_any_order(&mut have, &mut want);
        assert_eq!(result.missing, vec!["abc"]);
        assert_eq!(result.extra, vec![r"a\.c"]);
    }

    #[test]
    fn repeated_pattern_matches_one_line_per_copy() {
        let mut have = vec!["a1", "b", "a2"];
        let mut want = vec![r"a\d", "b", r"a\d"];
        assert!(compare_regexes_any_order(&mut have, &mut want).success());
    }

    #[test]
    fn repeated_pattern_requires_a_distinct_line() {
        let mut have = vec!["a1", "b"];
        let mut want = vec![r"a\d", "b", r"a\d"];
        let result = compare_regexes_any_order(&mut have, &mut want);
        assert!(!result.success());
        assert!(result.missing.is_empty());
        assert_eq!(result.extra, vec![r"a\d"]);
    }

    #[test]
    fn broad_pattern_does_not_consume_a_specific_lines_only_match() {
        let mut have = vec!["b", "c"];
        let mut want = vec![".*", "b"];
        assert!(compare_regexes_any_order(&mut have, &mut want).success());
    }

    #[test]
    fn reports_unmatched_line_and_pattern() {
        let mut have = vec!["alpha", "gamma"];
        let mut want = vec!["alpha", r"beta \d+"];
        let result = compare_regexes_any_order(&mut have, &mut want);
        assert_eq!(result.missing, vec!["gamma"]);
        assert_eq!(result.extra, vec![r"beta \d+"]);
    }

    #[test]
    #[should_panic(expected = "invalid regular expression '('")]
    fn invalid_regex() {
        let mut have = vec!["one"];
        let mut want = vec!["("];
        compare_regexes_any_order(&mut have, &mut want);
    }
}
