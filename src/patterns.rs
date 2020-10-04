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
extern crate regex;
use regex::RegexSet;
use std::collections::BTreeMap;

/// A category of handle format with their corresponding regex
/// TODO: Is it possible to map an enum directly to a Regex?
#[derive(Debug, Eq, PartialEq)]
enum HandlePattern {
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

const N_MOST_COMMON_WORDS: usize = 3;

/// Finds the most common words in a given search
pub(crate) fn get_most_common_words(
    search_results: &[egg_mode::search::SearchResult],
) -> BTreeMap<String, u32> {
    let mut map_word_to_count = BTreeMap::new();
    let mut total_words = 0;
    let mut tweets = 0;

    // Look thru the results
    for result in search_results {
        // Look thru each tweet
        tweets += result.statuses.len();

        for tweet in &result.statuses {
            // Normalize text (somewhat)
            let words = tweet.text.split_whitespace().collect::<Vec<&str>>();
            total_words += words.len();

            // Analyze each word
            for word in words {
                let normalized_word = word
                    .to_string()
                    .to_lowercase()
                    .replace(&['(', ')', ',', '\"', '.', ';', ':', '\''][..], "");
                if map_word_to_count.contains_key(&normalized_word) {
                    // Increment existing word
                    *map_word_to_count.get_mut(&normalized_word).unwrap() += 1;
                } else {
                    // Insert new word
                    map_word_to_count.insert(normalized_word.to_owned(), 1);
                }
            }
        }
    }

    println!(
        "Tweets:{} ,Total words: {}, unique words: {}, returning the {} most common ones",
        tweets,
        total_words,
        map_word_to_count.len(),
        N_MOST_COMMON_WORDS
    );
    map_word_to_count
        .into_iter()
        .take(N_MOST_COMMON_WORDS)
        .collect()
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
    let words = get_most_common_words(&[search.response]);
    assert_eq!(words.is_empty(), false);
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
