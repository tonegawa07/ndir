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
