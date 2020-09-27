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

use std::collections::BTreeMap;

fn count_word_occurance(search_result: &egg_mode::search::SearchResult) -> BTreeMap<String, u32> {
    let mut map_word_to_count = BTreeMap::new();

    // Look thru each tweet
    for tweet in &search_result.statuses {
        // Normalize text (somewhat)
        let words_in_tweet = tweet.text.to_lowercase();

        // Analyze each word
        for word in words_in_tweet.split_whitespace() {
            if map_word_to_count.contains_key(word) {
                // Increment existing word
                *map_word_to_count.get_mut(word).unwrap() += 1;
            } else {
                // Insert new word
                map_word_to_count.insert(word.to_owned(), 1);
            }
        }
    }

    map_word_to_count
}
const N_MOST_COMMON_WORDS: usize = 3;

pub(crate) fn get_most_common_words(search_result: &egg_mode::search::SearchResult) -> Vec<String> {
    let map_word_to_count = count_word_occurance(search_result);
    map_word_to_count
        .keys()
        .take(N_MOST_COMMON_WORDS)
        .cloned()
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
    let words = get_most_common_words(&search);
    assert_eq!(words.is_empty(), false);
}
