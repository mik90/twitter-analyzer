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
use regex::Regex;
use std::collections::BTreeMap;

/// A category of handle format with their corresponding regex
enum HandlePattern {
    NameWithNumbers, // somename1234514 [a-z]+\d+
    Lowercase,       // lowercase [a-z]+
    PascalCase,      // pascalCase [a-z]+[A-Z][a-z]+
    CamelCase,       // CamelCase [A-Z][a-z]+[A-Z][a-z]+
    Uppercase,       // UPPERCASE [A-Z]+
    Other,           // Other *
}

impl HandlePattern {
    /// Parse a handle into a category
    fn from(handle: &str) -> HandlePattern {
        // Create a regex to match against
        // https://stackoverflow.com/a/50520581
        handle_re = Regex::new(
            r#"(?x)
            ([a-z]+\d+) |
            (prev) |
            (goto)\s+(\d+)
        "#,
        )
        .unwrap();

        let captures = input_re.captures(input).map(|captures| {
            captures
                .iter() // All the captured groups
                .skip(1) // Skipping the complete match
                .flat_map(|c| c) // Ignoring all empty optional matches
                .map(|c| c.as_str()) // Grab the original strings
                .collect::<Vec<_>>() // Create a vector
        });

        // Match against the captured values as a slice
        match captures.as_ref().map(|c| c.as_slice()) {
            Some(["next"]) => current_question_number += 1,
            Some(["prev"]) => current_question_number -= 1,
            Some(["goto", x]) => {
                let x = x.parse().expect("can't parse number");
                current_question_number = x;
            }
            _ => panic!("Unknown Command: {}", input),
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
    let words = get_most_common_words(vec![search]);
    assert_eq!(words.is_empty(), false);
}
