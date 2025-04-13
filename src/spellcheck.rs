use rayon::prelude::*;
use serde_json;
use std::collections::{HashMap, HashSet};
use std::fs;

fn bounded_levenshtein(a: &str, b: &str, max_dist: usize) -> usize {
    let (shorter, longer) = if a.len() <= b.len() { (a, b) } else { (b, a) };
    if longer.len() - shorter.len() > max_dist {
        return max_dist + 1;
    }

    let n = longer.len();
    let mut prev: Vec<usize> = (0..=n).collect();
    let mut curr = vec![0; n + 1];
    let long = longer.as_bytes();

    for (i, &sc) in shorter.as_bytes().iter().enumerate() {
        let row = i + 1;
        curr[0] = row;

        let col_min = if row > max_dist { row - max_dist } else { 1 };
        let col_max = (row + max_dist).min(n);

        for j in 1..=n {
            if j < col_min || j > col_max {
                curr[j] = max_dist + 1;
                continue;
            }
            let cost = if sc == long[j - 1] { 0 } else { 1 };
            let ins = curr[j - 1] + 1;
            let del = prev[j] + 1;
            let sub = prev[j - 1] + cost;
            curr[j] = ins.min(del).min(sub);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[n]
}

fn deletion_variants(word: &str, max_del: usize, keep_original: bool) -> HashSet<String> {
    let mut seen = HashSet::new();
    if keep_original {
        seen.insert(word.to_owned());
    }
    let mut frontier: HashSet<String> = [word.to_owned()].into();

    for _ in 0..max_del {
        let mut next = HashSet::new();
        for variant in &frontier {
            let chars: Vec<char> = variant.chars().collect();
            for idx in 0..chars.len() {
                let mut shorter = String::with_capacity(variant.len());
                shorter.extend(chars[..idx].iter());
                shorter.extend(chars[idx + 1..].iter());
                if seen.insert(shorter.clone()) {
                    next.insert(shorter);
                }
            }
        }
        if next.is_empty() {
            break;
        }
        frontier = next;
    }
    seen
}

#[derive(Debug, Clone)]
pub struct Suggestion {
    pub word: String,
    pub distance: usize,
}

#[derive(Debug, Clone)]
pub enum SuggestedCorrection {
    NoSuggestions,
    Suggestions(Vec<Suggestion>),
}

#[derive(Debug, Clone)]
pub struct SpellCorrector {
    dictionary: HashSet<String>,
    dictionary_del_mappings: HashMap<String, Vec<String>>, // deletion edits -> correct words
    max_edit_distance: usize,                              // maximum edit distance to consider
}

impl SpellCorrector {
    pub fn new(dictionary: HashSet<String>, max_edit_distance: usize) -> Self {
        let mut dictionary_del_mappings = HashMap::new();
        for word in &dictionary {
            let deletions = deletion_variants(word, max_edit_distance, true);
            for del_word in &deletions {
                dictionary_del_mappings
                    .entry(del_word.clone())
                    .or_insert_with(Vec::new)
                    .push(word.clone());
            }
        }
        SpellCorrector {
            dictionary,
            dictionary_del_mappings,
            max_edit_distance,
        }
    }

    pub fn from_word_list_file(file_path: &str, max_edit_distance: usize) -> Self {
        let content = fs::read_to_string(file_path).expect("Unable to read dictionary file");
        let dictionary: HashSet<String> = content
            .lines()
            .map(|s| s.to_string().to_lowercase())
            .collect();
        Self::new(dictionary, max_edit_distance)
    }

    pub fn save_spell_corrector(&self, file_path: &str) -> Result<(), std::io::Error> {
        let del_mapping_json = serde_json::to_string(&self.dictionary_del_mappings);

        match del_mapping_json {
            Ok(json) => {
                fs::write(file_path, json)?;
                Ok(())
            }
            Err(e) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to serialize dictionary mappings: {}", e),
            )),
        }
    }

    pub fn load_spell_corrector(file_path: &str, max_edit_distance: usize) -> Self {
        let content = fs::read_to_string(file_path).expect("Unable to read dictionary file");
        let dictionary_del_mappings: HashMap<String, Vec<String>> =
            serde_json::from_str(&content).expect("Failed to deserialize dictionary mappings");
        let mut dictionary = HashSet::new();
        for words in dictionary_del_mappings.values() {
            for word in words {
                dictionary.insert(word.clone());
            }
        }
        SpellCorrector {
            dictionary,
            dictionary_del_mappings,
            max_edit_distance,
        }
    }

    pub fn add_word_to_dictionary(&mut self, word: &str) {
        self.dictionary.insert(word.to_string());
        let deletions = deletion_variants(word, self.max_edit_distance, true);
        for del_word in &deletions {
            self.dictionary_del_mappings
                .entry(del_word.clone())
                .or_insert_with(Vec::new)
                .push(word.to_string());
        }
    }

    pub fn suggest_single_word_corrections(
        &self,
        word: &str,
        n_suggestions: usize,
    ) -> SuggestedCorrection {
        if self.dictionary.contains(word) {
            return SuggestedCorrection::NoSuggestions;
        }

        let word_deletions = deletion_variants(word, self.max_edit_distance, false);
        let mut candidates = HashSet::new();

        for del_word in &word_deletions {
            if let Some(words) = self.dictionary_del_mappings.get(del_word) {
                candidates.extend(words.iter().cloned());
            }
        }

        let mut suggestions: Vec<Suggestion> = candidates
            .into_iter()
            .filter_map(|candidate| {
                let distance = bounded_levenshtein(word, &candidate, self.max_edit_distance);
                if distance <= self.max_edit_distance {
                    Some(Suggestion {
                        word: candidate,
                        distance,
                    })
                } else {
                    None
                }
            })
            .collect();

        suggestions.sort_by(|a, b| {
            a.distance
                .cmp(&b.distance)
                .then_with(|| b.word.len().cmp(&a.word.len()))
                .then_with(|| a.word.cmp(&b.word))
        });

        suggestions.truncate(n_suggestions);

        SuggestedCorrection::Suggestions(suggestions.into_iter().collect())
    }

    pub fn suggest_word_corrections(
        &self,
        words: &Vec<String>,
        n_suggestions: usize,
    ) -> Vec<SuggestedCorrection> {
        words
            // .into_iter()
            .par_iter()
            .map(|word| self.suggest_single_word_corrections(&word, n_suggestions))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounded_levenshtein() {
        assert_eq!(bounded_levenshtein("kitten", "sitting", 3), 3);
        assert_eq!(bounded_levenshtein("flaw", "lawn", 2), 2);
        assert_eq!(bounded_levenshtein("intention", "execution", 5), 5);
    }

    #[test]
    fn test_deletion_variants() {
        let variants = deletion_variants("spelling", 2, false);
        assert!(variants.contains("speling"));
        assert!(variants.contains("speling"));
        assert!(!variants.contains("spelling"));
    }

    #[test]
    fn test_suggest_single_word_corrections() {
        let mut dictionary: HashSet<String> = HashSet::new();
        dictionary.insert("spelling".to_string());
        dictionary.insert("corrected".to_string());

        let spell_corrector = SpellCorrector::new(dictionary, 2);
        let suggestions = spell_corrector.suggest_single_word_corrections("speling", 2);

        match suggestions {
            SuggestedCorrection::NoSuggestions => panic!("Expected suggestions, but got none."),
            SuggestedCorrection::Suggestions(suggestions) => {
                assert_eq!(suggestions.len(), 1);
                assert_eq!(suggestions[0].word, "spelling");
            }
        }
    }

    #[test]
    fn test_bounded_levenshtein_identical() {
        assert_eq!(bounded_levenshtein("same", "same", 0), 0);
    }

    #[test]
    fn test_bounded_levenshtein_cutoff() {
        // real distance = 3, bound = 2  ⇒  function must bail out (> bound)
        assert!(bounded_levenshtein("kitten", "sitting", 2) > 2);
    }

    #[test]
    fn test_deletion_variants_zero() {
        // With max_del = 0 we expect *no* variants
        assert!(deletion_variants("abc", 0, false).is_empty());
    }

    #[test]
    fn test_deletion_variants_exhaustive() {
        let v = deletion_variants("abc", 2, false);
        // All unique strings reachable by 1 or 2 deletions
        for s in ["ab", "ac", "bc", "a", "b", "c"] {
            assert!(v.contains(s), "missing variant {}", s);
        }
        // Original word must be absent
        assert!(!v.contains("abc"));
    }

    #[test]
    fn test_suggest_multiple_corrections_order() {
        let dict: HashSet<String> = ["spelling", "spilling", "selling"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let corrector = SpellCorrector::new(dict, 2);

        println!(
            "Res: {:?}",
            corrector.suggest_single_word_corrections("speling", 2)
        );

        match corrector.suggest_single_word_corrections("speling", 2) {
            SuggestedCorrection::Suggestions(list) => {
                assert_eq!(list[0].word, "spelling"); // distance 1
                assert_eq!(list[1].word, "spilling"); // distance 2
                assert_eq!(list.len(), 2);
            }
            _ => panic!("expected suggestions"),
        }
    }

    #[test]
    fn test_add_word_updates_dictionary() {
        let dict: HashSet<String> = ["cat"].iter().map(|s| s.to_string()).collect();
        let mut corrector = SpellCorrector::new(dict, 1);

        println!(
            "Res: {:?}",
            corrector.suggest_single_word_corrections("cart", 2)
        );
        // "cart" is unknown → only "cat" suggested
        if let SuggestedCorrection::Suggestions(list) =
            corrector.suggest_single_word_corrections("cart", 2)
        {
            assert_eq!(list[0].word, "cat");
        }

        // Add "cart", run again – now no suggestions (exact hit)
        corrector.add_word_to_dictionary("cart");
        match corrector.suggest_single_word_corrections("cart", 2) {
            SuggestedCorrection::NoSuggestions => {}
            _ => panic!("expected no suggestions after adding exact word"),
        }
    }
}
