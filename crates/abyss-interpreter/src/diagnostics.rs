//! Lightweight helpers shared between error-reporting sites.
//!
//! The interpreter raises errors as `EvalError` strings; this module supplies
//! the bits of formatting reused across several call sites — currently the
//! Levenshtein-based "did you mean?" suggestion logic that enriches messages
//! about undefined identifiers (variables and functions). The same primitives
//! are intended to be reused as more diagnostic flavours land (methods,
//! artifact fields).

/// Compute the Levenshtein edit distance between two strings.
///
/// O(len(a) * len(b)) time, O(min(len)) space — adequate for short identifier
/// names. Used by [`did_you_mean`].
pub(crate) fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr: Vec<usize> = vec![0; b.len() + 1];
    for i in 1..=a.len() {
        curr[0] = i;
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1) // deletion
                .min(curr[j - 1] + 1) // insertion
                .min(prev[j - 1] + cost); // substitution
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b.len()]
}

/// Pick up to `max_count` candidate names whose Levenshtein distance from
/// `target` is small enough to plausibly be a typo. The threshold scales with
/// `target` length (longer names tolerate slightly more edits) but is at most
/// half the length so vastly different names never surface as suggestions.
///
/// Returns names in ascending distance order; ties are broken alphabetically
/// so the result is deterministic even when the candidate iterator (often a
/// `HashMap::keys()`) yields entries in random order. The exact `target` is
/// filtered out — no point suggesting "did you mean: X?" when the user
/// already typed X.
pub(crate) fn did_you_mean<'a, I>(target: &str, candidates: I, max_count: usize) -> Vec<String>
where
    I: IntoIterator<Item = &'a str>,
{
    if max_count == 0 {
        return Vec::new();
    }
    let target_len = target.chars().count();
    // Allow more edits for longer names, but never more than half the length —
    // beyond that the candidate is too dissimilar to be worth surfacing.
    let max_distance = ((target_len / 3).max(2)).min(target_len.div_ceil(2).max(1));

    let mut scored: Vec<(usize, String)> = candidates
        .into_iter()
        .filter(|name| *name != target)
        .map(|name| (levenshtein(target, name), name.to_string()))
        .filter(|(dist, _)| *dist <= max_distance)
        .collect();
    // Tie-break alphabetically on the name so the final ordering is
    // deterministic regardless of the candidate-iterator order. Avoids flaky
    // output when candidates come from a `HashMap`.
    scored.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    scored.dedup_by(|a, b| a.1 == b.1);
    scored.truncate(max_count);
    scored.into_iter().map(|(_, name)| name).collect()
}

/// Append a parenthesised "did you mean: …" hint to `name` when at least one
/// suggestion is available. Renders as:
///
/// - one suggestion: `"X"`
/// - two suggestions: `"X or Y"`
/// - three or more: `"X, Y, or Z"` (Oxford comma followed by `"or"`)
///
/// When the suggestion list is empty the original `name` is returned
/// unchanged, so call sites can hand the result straight to
/// `EvalError::UndefinedVariable(...)` without further branching.
pub(crate) fn label_with_suggestions(name: &str, suggestions: &[String]) -> String {
    match suggestions {
        [] => name.to_string(),
        [only] => format!("{} (did you mean: {}?)", name, only),
        [first, second] => format!("{} (did you mean: {} or {}?)", name, first, second),
        many => {
            // Three or more — comma-separated with an Oxford comma before
            // the final `"or"`, e.g. "a, b, or c".
            let (last, rest) = many.split_last().expect("non-empty in this arm");
            format!("{} (did you mean: {}, or {}?)", name, rest.join(", "), last)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levenshtein_handles_empty_inputs() {
        assert_eq!(levenshtein("", ""), 0);
        assert_eq!(levenshtein("forge", ""), 5);
        assert_eq!(levenshtein("", "forge"), 5);
    }

    #[test]
    fn levenshtein_matches_known_distances() {
        assert_eq!(levenshtein("forge", "forge"), 0);
        assert_eq!(levenshtein("forge", "forg"), 1);
        assert_eq!(levenshtein("counter", "countar"), 1);
        assert_eq!(levenshtein("kitten", "sitting"), 3);
    }

    #[test]
    fn did_you_mean_returns_closest_first() {
        let candidates = ["counter", "countdown", "cosine"];
        assert_eq!(did_you_mean("countar", candidates, 3), vec!["counter"]);
    }

    #[test]
    fn did_you_mean_skips_exact_match() {
        let candidates = ["counter", "countdown"];
        assert_eq!(did_you_mean("counter", candidates, 3), Vec::<String>::new());
    }

    #[test]
    fn did_you_mean_respects_max_count() {
        let candidates = ["counter", "counters", "counted"];
        assert_eq!(did_you_mean("countet", candidates, 2).len(), 2);
    }

    #[test]
    fn did_you_mean_filters_distant_candidates() {
        let candidates = ["completely_different", "another_unrelated"];
        assert!(did_you_mean("xyz", candidates, 3).is_empty());
    }

    #[test]
    fn label_with_suggestions_formats_one_match() {
        assert_eq!(
            label_with_suggestions("countar", &["counter".to_string()]),
            "countar (did you mean: counter?)"
        );
    }

    #[test]
    fn label_with_suggestions_formats_two_matches() {
        assert_eq!(
            label_with_suggestions("healht", &["health".to_string(), "heat".to_string()]),
            "healht (did you mean: health or heat?)"
        );
    }

    #[test]
    fn label_with_suggestions_formats_three_or_more_matches_with_oxford_comma() {
        assert_eq!(
            label_with_suggestions(
                "countet",
                &[
                    "counter".to_string(),
                    "counted".to_string(),
                    "counters".to_string(),
                ]
            ),
            "countet (did you mean: counter, counted, or counters?)"
        );
    }

    #[test]
    fn label_with_suggestions_passes_through_when_empty() {
        assert_eq!(label_with_suggestions("countar", &[]), "countar");
    }

    #[test]
    fn did_you_mean_breaks_ties_alphabetically_for_determinism() {
        // Two candidates at distance 1 from "abz": "abx" and "abc".
        // Regardless of input order, alphabetical tie-break gives "abc" first.
        assert_eq!(did_you_mean("abz", ["abx", "abc"], 2), vec!["abc", "abx"]);
        assert_eq!(did_you_mean("abz", ["abc", "abx"], 2), vec!["abc", "abx"]);
    }
}
