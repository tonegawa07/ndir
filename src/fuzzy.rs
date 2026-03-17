use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

pub struct FuzzyFilter {
    matcher: SkimMatcherV2,
}

impl FuzzyFilter {
    pub fn new() -> Self {
        Self {
            matcher: SkimMatcherV2::default(),
        }
    }

    /// Filter entries by query, returning (index, score) pairs sorted by score descending.
    /// If query is empty, returns all indices with score 0.
    pub fn filter(&self, query: &str, entries: &[String]) -> Vec<(usize, i64)> {
        if query.is_empty() {
            return entries.iter().enumerate().map(|(i, _)| (i, 0)).collect();
        }

        let mut results: Vec<(usize, i64)> = entries
            .iter()
            .enumerate()
            .filter_map(|(i, name)| {
                self.matcher
                    .fuzzy_match(name, query)
                    .map(|score| (i, score))
            })
            .collect();

        results.sort_by(|a, b| b.1.cmp(&a.1));
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_query_returns_all() {
        let f = FuzzyFilter::new();
        let entries = vec!["foo".into(), "bar".into(), "baz".into()];
        let result = f.filter("", &entries);
        assert_eq!(result.len(), 3);
        assert!(result.iter().all(|(_, score)| *score == 0));
    }

    #[test]
    fn matching_entries_are_returned() {
        let f = FuzzyFilter::new();
        let entries = vec!["abcxyz".into(), "abc".into(), "xabc".into(), "zzz".into()];
        let result = f.filter("abc", &entries);
        assert_eq!(result.len(), 3);
        let indices: Vec<usize> = result.iter().map(|(i, _)| *i).collect();
        assert!(indices.contains(&0));
        assert!(indices.contains(&1));
        assert!(indices.contains(&2));
        assert!(!indices.contains(&3));
    }

    #[test]
    fn no_match_returns_empty() {
        let f = FuzzyFilter::new();
        let entries = vec!["foo".into(), "bar".into()];
        let result = f.filter("zzz", &entries);
        assert!(result.is_empty());
    }

    #[test]
    fn results_sorted_by_score_descending() {
        let f = FuzzyFilter::new();
        let entries = vec!["xsrcx".into(), "src".into(), "my_src_dir".into()];
        let result = f.filter("src", &entries);
        for w in result.windows(2) {
            assert!(w[0].1 >= w[1].1);
        }
    }

    #[test]
    fn case_insensitive_matching() {
        let f = FuzzyFilter::new();
        let entries = vec!["README".into(), "src".into()];
        let result = f.filter("readme", &entries);
        assert!(!result.is_empty());
        assert_eq!(result[0].0, 0);
    }
}
