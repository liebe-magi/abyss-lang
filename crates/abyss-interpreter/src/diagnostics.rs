//! Lightweight helpers shared between error-reporting sites.
//!
//! The interpreter raises errors as `EvalError` strings; this module supplies
//! the bits of formatting reused across several call sites — chiefly the
//! Levenshtein-based "did you mean?" suggestion logic that enriches messages
//! about undefined identifiers (variables, functions, methods, and artifact
//! fields). Two rendering modes are exposed so callers can pick the form that
//! reads best in their message:
//!
//! - [`label_with_suggestions`] — appends the hint after a bare identifier
//!   (`foo (did you mean: bar?)`); used where the identifier is the whole
//!   message subject (e.g. `EvalError::UndefinedVariable`).
//! - [`format_suggestions_hint`] / [`did_you_mean_hint`] — return just the
//!   parenthesised suffix so the caller can splice it inside a longer
//!   message that already wraps the identifier in punctuation
//!   (`Field 'fild' (did you mean: field?) does not exist on …`).

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

/// Render just the parenthesised "did you mean: …" suffix — without any
/// preceding identifier. Returns `None` when the suggestion list is empty so
/// callers can decide where to splice the hint into a larger message (e.g.
/// outside of single quotes around a field name).
///
/// Format mirrors [`label_with_suggestions`]:
/// - one suggestion: `"(did you mean: X?)"`
/// - two suggestions: `"(did you mean: X or Y?)"`
/// - three or more: `"(did you mean: X, Y, or Z?)"` (Oxford comma)
pub(crate) fn format_suggestions_hint(suggestions: &[String]) -> Option<String> {
    match suggestions {
        [] => None,
        [only] => Some(format!("(did you mean: {}?)", only)),
        [first, second] => Some(format!("(did you mean: {} or {}?)", first, second)),
        many => {
            let (last, rest) = many.split_last().expect("non-empty in this arm");
            Some(format!("(did you mean: {}, or {}?)", rest.join(", "), last))
        }
    }
}

/// Append a parenthesised "did you mean: …" hint to `name` when at least one
/// suggestion is available. When the suggestion list is empty the original
/// `name` is returned unchanged, so call sites can hand the result straight to
/// an `EvalError` variant without further branching.
pub(crate) fn label_with_suggestions(name: &str, suggestions: &[String]) -> String {
    match format_suggestions_hint(suggestions) {
        None => name.to_string(),
        Some(hint) => format!("{} {}", name, hint),
    }
}

/// Convenience composition of [`did_you_mean`] and
/// [`format_suggestions_hint`]: pick suggestions from `candidates` and return
/// just the parenthesised "(did you mean: …)" suffix (or `None`).
///
/// Used when the call site needs to splice the hint into a larger message
/// that already wraps the target identifier — for example, after a closing
/// single quote around a field name in
/// `Field 'fild' (did you mean: field?) does not exist on …`.
pub(crate) fn did_you_mean_hint<'a, I>(
    target: &str,
    candidates: I,
    max_count: usize,
) -> Option<String>
where
    I: IntoIterator<Item = &'a str>,
{
    let suggestions = did_you_mean(target, candidates, max_count);
    format_suggestions_hint(&suggestions)
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

    #[test]
    fn did_you_mean_hint_renders_parenthetical_when_close_match_exists() {
        assert_eq!(
            did_you_mean_hint("fild", ["name", "field", "level"], 3).as_deref(),
            Some("(did you mean: field?)")
        );
    }

    #[test]
    fn did_you_mean_hint_returns_none_when_no_close_match() {
        assert!(did_you_mean_hint("xyz", ["name", "field"], 3).is_none());
    }

    #[test]
    fn format_suggestions_hint_returns_none_for_empty() {
        assert!(format_suggestions_hint(&[]).is_none());
    }

    #[test]
    fn format_suggestions_hint_renders_two_with_or() {
        assert_eq!(
            format_suggestions_hint(&["health".to_string(), "heat".to_string()]).as_deref(),
            Some("(did you mean: health or heat?)")
        );
    }

    #[test]
    fn format_suggestions_hint_renders_three_or_more_with_oxford_comma() {
        assert_eq!(
            format_suggestions_hint(&[
                "counter".to_string(),
                "counted".to_string(),
                "counters".to_string()
            ])
            .as_deref(),
            Some("(did you mean: counter, counted, or counters?)")
        );
    }
}
