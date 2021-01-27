extern crate chrono;
extern crate regex;
use crate::{
    storage::{self, StorageHandler},
    twitter::QueryResult,
};
use regex::RegexSet;
use std::{collections::BTreeMap, io, iter::FromIterator, path::Path};

/// Result of examining account
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct SearchAnalysis {
    pub queries: Vec<String>,
    pub date_utc: chrono::DateTime<chrono::Utc>,
    pub word_frequency: Vec<(String, usize)>,
    pub handle_patterns: Vec<(HandlePattern, usize)>,
}

/// Analysis configuration
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct AnalysisConfig {
    pub version: u32,
    pub description: String,
    pub ignored_words: Vec<String>,
}

impl AnalysisConfig {
    pub fn new(config_path: &std::path::Path) -> Option<AnalysisConfig> {
        // TODO Map the io::Result or serde_json::Result to the same type
        let file_string = std::fs::read_to_string(config_path);
        if file_string.is_err() {
            eprintln!(
                "Could not read {:?}, got error:{:?}",
                config_path,
                file_string.err()
            );
            return None;
        }
        let deserialied_json: serde_json::Result<AnalysisConfig> =
            serde_json::from_str(file_string.unwrap().as_str());
        if deserialied_json.is_err() {
            eprintln!("serde_json parse error: {:?}", deserialied_json.err());
            return None;
        }

        Some(deserialied_json.unwrap())
    }
}

const N_MOST_COMMON_WORDS: usize = 5;
const N_MOST_HANDLE_PATTERNS: usize = 3;

impl SearchAnalysis {
    pub fn from_stored_queries(
        base_dir: &Path,
        words_to_ignore: &[String],
    ) -> io::Result<SearchAnalysis> {
        let query_results = StorageHandler::new()
            .storage_dir(&base_dir)
            .retrieve_all_queries()?;
        Ok(SearchAnalysis {
            queries: query_results.iter().map(|x| x.query.to_string()).collect(),
            date_utc: chrono::Utc::now(),
            word_frequency: get_most_common_words(&query_results, &words_to_ignore),
            handle_patterns: get_most_common_handle_patterns(&query_results),
        })
    }

    pub fn summary(&self) -> String {
        let mut summary = String::from("------------------------------------\n");

        summary.push_str(format!("Most common words for queries: {:?}\n", self.queries).as_str());

        for word in self.word_frequency.iter().take(N_MOST_COMMON_WORDS) {
            summary.push_str(format!("{} was seen {} times\n", word.0, word.1).as_str());
        }

        for pattern in self.handle_patterns.iter().take(N_MOST_HANDLE_PATTERNS) {
            summary.push_str(
                format!("The pattern {:?} was seen {} times\n", pattern.0, pattern.1).as_str(),
            );
        }
        summary.push_str("------------------------------------\n");

        summary
    }
}

pub async fn run_analysis_with_config(config: AnalysisConfig) -> io::Result<()> {
    let analysis = SearchAnalysis::from_stored_queries(
        &Path::new(storage::DEFAULT_STORAGE_DIR),
        &config.ignored_words,
    )?;
    let storage = StorageHandler::new();
    storage.save_analysis(&analysis)?;
    println!("{}", analysis.summary());
    Ok(())
}

/**
 *  A category of handle format with their corresponding regex
 *
 * - Patterns:
 *      - Most common words used in replies
 *      - Most common username format
 *          - somename1234514
 *          - FirstnameLastname
 *          - lowercase
 *          - PascalCase
 *          - CamelCase
 *          - UPPERCASE
 *          - Other
 *      - Account age
 *      - Account location
 */
/// TODO: Is it possible to map an enum directly to a Regex?
#[derive(serde::Serialize, serde::Deserialize, Debug, Eq, PartialEq, PartialOrd, Ord, Clone)]
pub enum HandlePattern {
    NameWithNumbers = 0, // somename1234514 [a-z]+\d+
    Lowercase = 1,       // lowercase [a-z]+
    PascalCase = 2,      // pascalCase [a-z]+[A-Z][a-z]+
    CamelCase = 3,       // CamelCase [A-Z][a-z]+[A-Z][a-z]+
    Uppercase = 4,       // UPPERCASE [A-Z]+
    Other = 5,           // Other .*
}

impl HandlePattern {
    /// Parse a handle into a category
    pub fn from(handle: &str) -> HandlePattern {
        let set = RegexSet::new(&[
            r"^[a-z]+\d+$",              // NameWithNumbers
            r"^[a-z]+$",                 // Lowercase
            r"^[a-z]+[A-Z][a-z]+$",      // pascalCase
            r"^[A-Z][a-z]+[A-Z][a-z]+$", // CamelCase
            r"^[A-Z]+$",                 // Uppercase
            r"^.*$",                     // Other
        ])
        .unwrap();
        let matches = set.matches(handle);

        if matches.matched(HandlePattern::NameWithNumbers as usize) {
            HandlePattern::NameWithNumbers
        } else if matches.matched(HandlePattern::Lowercase as usize) {
            HandlePattern::Lowercase
        } else if matches.matched(HandlePattern::PascalCase as usize) {
            HandlePattern::PascalCase
        } else if matches.matched(HandlePattern::CamelCase as usize) {
            HandlePattern::CamelCase
        } else if matches.matched(HandlePattern::Uppercase as usize) {
            HandlePattern::Uppercase
        } else {
            HandlePattern::Other
        }
    }
}

/// Finds the most common words in a given search
pub fn get_most_common_words(
    query_results: &[QueryResult],
    ignored_words: &[String],
) -> Vec<(String, usize)> {
    let mut map_word_to_count: BTreeMap<String, usize> = BTreeMap::new();

    for query in query_results {
        for tweet in &query.tweets {
            // Normalize text (somewhat)
            let words = tweet.text.split_whitespace().collect::<Vec<&str>>();

            // Analyze each word
            for word in words {
                let normalized_word = word
                    .to_string()
                    .to_lowercase()
                    .replace(&['(', ')', ',', '\"', '.', ';', ':', '\'', '!'][..], "");
                if !ignored_words.contains(&normalized_word) {
                    // Insert count of 0 if the word was not seen before
                    *map_word_to_count.entry(normalized_word).or_insert(0) += 1;
                }
            }
        }
    }

    // https://stackoverflow.com/questions/41220872/how-if-possible-to-sort-a-btreemap-by-value-in-rust
    let mut sorted_values = Vec::from_iter(map_word_to_count);
    // Count should be in decreasing order
    sorted_values.sort_unstable_by(|&(_, a), &(_, b)| b.cmp(&a));
    sorted_values
}

/// Finds the most common handle patterns in a given search
pub fn get_most_common_handle_patterns(
    query_results: &[QueryResult],
) -> Vec<(HandlePattern, usize)> {
    let mut map_pattern_to_count: BTreeMap<HandlePattern, usize> = BTreeMap::new();

    for query in query_results {
        for tweet in &query.tweets {
            let handle = &tweet.handle;
            let pattern = HandlePattern::from(handle.as_str());

            // Insert count of 0 if the pattern was not seen before
            *map_pattern_to_count.entry(pattern).or_insert(0) += 1;
        }
    }
    let mut sorted_values = Vec::from_iter(map_pattern_to_count);
    // Count should be in decreasing order
    sorted_values.sort_unstable_by(|&(_, a), &(_, b)| b.cmp(&a));
    sorted_values
}

#[cfg(test)]
mod test {
    use super::{get_most_common_handle_patterns, get_most_common_words, HandlePattern};
    use crate::twitter::QueryResult;
    use crate::util::test::get_test_query_result;
    use std::cmp::Ordering;

    #[tokio::test]
    async fn test_most_common_words() {
        let queries: Vec<QueryResult> = vec![get_test_query_result()];
        let words = get_most_common_words(&queries, &Vec::new());
        assert!(!words.is_empty());
    }

    #[tokio::test]
    // Note: for this test, ensure that the test query has enough repeated words to be usable!
    async fn test_most_common_words_order() {
        let queries: Vec<QueryResult> = vec![get_test_query_result()];
        let words = get_most_common_words(&queries, &Vec::new());
        assert_eq!(words.is_empty(), false);

        println!("Words: {:?}", words);
        // Check ordering of elements, the earlier items should be greater than the succeeding ones
        // Reference: https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.partial_cmp
        assert_eq!(
            words[0..].iter().partial_cmp(&words[1..]),
            Some(Ordering::Greater)
        );
    }

    #[tokio::test]
    async fn test_handle_patterns() {
        let queries: Vec<QueryResult> = vec![get_test_query_result()];
        let patterns = get_most_common_handle_patterns(&queries);
        assert!(!patterns.is_empty());
    }

    #[tokio::test]
    async fn test_handle_patterns_order() {
        let queries: Vec<QueryResult> = vec![get_test_query_result()];
        let patterns = get_most_common_handle_patterns(&queries);
        assert_eq!(
            patterns[0..].iter().partial_cmp(&patterns[1..]),
            Some(Ordering::Greater)
        );
    }

    #[tokio::test]
    async fn test_handle_firstname_lastname() {
        assert_eq!(
            HandlePattern::from("firstname1234"),
            HandlePattern::NameWithNumbers
        );
    }

    #[tokio::test]
    async fn test_handle_lowercase() {
        assert_eq!(HandlePattern::from("lowercase"), HandlePattern::Lowercase);
    }

    #[tokio::test]
    async fn test_handle_uppercase() {
        assert_eq!(HandlePattern::from("UPPERCASE"), HandlePattern::Uppercase);
    }

    #[tokio::test]
    async fn test_handle_camelcase() {
        assert_eq!(HandlePattern::from("CamelCase"), HandlePattern::CamelCase);
    }

    #[tokio::test]
    async fn test_handle_pascalcase() {
        assert_eq!(HandlePattern::from("pascalCase"), HandlePattern::PascalCase);
    }

    #[tokio::test]
    async fn test_handle_other() {
        assert_eq!(HandlePattern::from("123o%her"), HandlePattern::Other);
    }
}
