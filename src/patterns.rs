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
use regex::RegexSet;
use std::collections::BTreeMap;

/// Result of examining account
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct SearchAnalysis {
    query: String,
    search_date_utc: chrono::DateTime<chrono::Utc>,
    word_frequency: BTreeMap<String, usize>,
    handle_patterns: BTreeMap<HandlePattern, usize>,
}

impl SearchAnalysis {
    pub fn new(
        query: &str,
        date: chrono::DateTime<chrono::Utc>,
        search: &egg_mode::search::SearchResult,
    ) -> Option<SearchAnalysis> {
        Some(SearchAnalysis {
            query: query.to_owned(),
            search_date_utc: date,
            word_frequency: get_most_common_words(&search),
            handle_patterns: get_most_common_handle_patterns(&search),
        })
    }

    fn get_unique_words_seen(&self) -> usize {
        self.word_frequency.len()
    }

    fn get_handle_patterns_seen(&self) -> usize {
        self.handle_patterns.len()
    }
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
pub(crate) fn get_most_common_words(
    search_result: &egg_mode::search::SearchResult,
) -> BTreeMap<String, usize> {
    let mut map_word_to_count = BTreeMap::new();

    for tweet in &search_result.statuses {
        // Normalize text (somewhat)
        let words = tweet.text.split_whitespace().collect::<Vec<&str>>();

        // Analyze each word
        for word in words {
            let normalized_word = word
                .to_string()
                .to_lowercase()
                .replace(&['(', ')', ',', '\"', '.', ';', ':', '\''][..], "");
            // Insert count of 0 if the word was not seen before
            *map_word_to_count.entry(normalized_word).or_insert(0) += 1;
        }
    }

    map_word_to_count
}

/// Finds the most common handle patterns in a given search
pub(crate) fn get_most_common_handle_patterns(
    search_result: &egg_mode::search::SearchResult,
) -> BTreeMap<HandlePattern, usize> {
    let mut map_pattern_to_count = BTreeMap::new();

    for tweet in &search_result.statuses {
        let handle = &tweet.user.as_ref().unwrap().screen_name;
        let pattern = HandlePattern::from(handle.as_str());

        // Insert count of 0 if the pattern was not seen before
        *map_pattern_to_count.entry(pattern).or_insert(0) += 1;
    }

    map_pattern_to_count
}

#[tokio::test]
async fn test_most_common_words() {
    let token = crate::auth::get_token(std::path::Path::new("auth/bearer.token")).unwrap();
    let search = egg_mode::search::search("twitter")
        .result_type(egg_mode::search::ResultType::Recent)
        .count(1)
        .call(&token)
        .await
        .unwrap();
    let words = get_most_common_words(&search.response);
    assert!(!words.is_empty());
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
