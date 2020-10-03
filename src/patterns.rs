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

const N_MOST_COMMON_WORDS: usize = 3;

pub(crate) fn get_most_common_words(
    search_results: &[egg_mode::search::SearchResult],
) -> BTreeMap<String, u32> {
    let mut map_word_to_count = BTreeMap::new();
    let mut total_words = 0;
    let mut tweets = 0;

    // Look thru the results
    for status in search_results {
        // Look thru each tweet
        for tweet in &status.statuses {
            tweets += 1;
            // Normalize text (somewhat)
            let words_in_tweet = tweet.text.to_lowercase();

            // Analyze each word
            for word in words_in_tweet.split_whitespace() {
                total_words += 1;
                if map_word_to_count.contains_key(word) {
                    // Increment existing word
                    *map_word_to_count.get_mut(word).unwrap() += 1;
                } else {
                    // Insert new word
                    map_word_to_count.insert(word.to_owned(), 1);
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
