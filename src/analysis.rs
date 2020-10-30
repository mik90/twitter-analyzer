/*
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
 * - Serialize summation to disk in json
 */
extern crate chrono;
extern crate regex;
use crate::{storage, twitter::QueryResult};
use regex::RegexSet;
use std::{
    collections::BTreeMap,
    io,
    path::{Path, PathBuf},
};

/// Result of examining account
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct SearchAnalysis {
    pub queries: Vec<String>,
    pub date_utc: chrono::DateTime<chrono::Utc>,
    pub word_frequency: BTreeMap<String, usize>,
    pub handle_patterns: BTreeMap<HandlePattern, usize>,
}

const N_MOST_COMMON_WORDS: usize = 5;
const N_MOST_HANDLE_PATTERNS: usize = 3;

impl SearchAnalysis {
    pub fn from_stored_queries(base_dir: &Path, queries: Vec<&str>) -> io::Result<SearchAnalysis> {
        let query_results = storage::retrieve_queries(base_dir, &queries)?;
        Ok(SearchAnalysis {
            word_frequency: get_most_common_words(&query_results),
            handle_patterns: get_most_common_handle_patterns(&query_results),
            date_utc: chrono::Utc::now(),
            queries: query_results.iter().map(|x| x.query.to_string()).collect(),
        })
    }

    #[allow(dead_code)]
    pub fn create_empty() -> SearchAnalysis {
        SearchAnalysis {
            queries: Vec::new(),
            word_frequency: BTreeMap::new(),
            date_utc: chrono::Utc::now(),
            handle_patterns: BTreeMap::new(),
        }
    }

    /// Saves to $PWD/<base_dir>/<handle>/<search-date>/analysis.json
    pub fn storage_location(&self, base_dir: &Path) -> PathBuf {
        // ISO 8601 / RFC 3339 date & time format
        let mut path = PathBuf::from(base_dir);
        path.push(&self.date_utc.format("%+").to_string());
        path.push("analysis.json");
        path
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

pub(crate) async fn run_analysis(queries: Vec<&str>) -> io::Result<()> {
    // TODO run analysis from dir
    let analysis = SearchAnalysis::from_stored_queries(
        &Path::new(storage::DEFAULT_QUERY_RESULT_DIR),
        queries,
    )?;
    storage::store_analysis(&analysis)?;
    println!("{}", analysis.summary());
    Ok(())
}

/// A category of handle format with their corresponding regex
/// TODO: Is it possible to map an enum directly to a Regex?
#[derive(serde::Serialize, serde::Deserialize, Debug, Eq, PartialEq, PartialOrd, Ord)]
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
pub(crate) fn get_most_common_words(query_results: &[QueryResult]) -> BTreeMap<String, usize> {
    let mut map_word_to_count = BTreeMap::new();

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

                // Insert count of 0 if the word was not seen before
                *map_word_to_count.entry(normalized_word).or_insert(0) += 1;
            }
        }
    }

    map_word_to_count
}

/// Finds the most common handle patterns in a given search
pub(crate) fn get_most_common_handle_patterns(
    query_results: &[QueryResult],
) -> BTreeMap<HandlePattern, usize> {
    let mut map_pattern_to_count = BTreeMap::new();

    for query in query_results {
        for tweet in &query.tweets {
            let handle = &tweet.handle;
            let pattern = HandlePattern::from(handle.as_str());

            // Insert count of 0 if the pattern was not seen before
            *map_pattern_to_count.entry(pattern).or_insert(0) += 1;
        }
    }
    map_pattern_to_count
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_most_common_words() {
        let queries: Vec<QueryResult> = vec![crate::test::get_test_query_result()];
        let words = get_most_common_words(&queries);
        assert!(!words.is_empty());
    }

    #[tokio::test]
    async fn test_handle_patterns() {
        let queries: Vec<QueryResult> = vec![crate::test::get_test_query_result()];
        let patterns = get_most_common_handle_patterns(&queries);
        assert!(!patterns.is_empty());
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
