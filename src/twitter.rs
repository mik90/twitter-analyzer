use crate::storage::StorageHandler;
use std::{fs, io, path::PathBuf};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Tweet {
    pub text: String,
    pub handle: String,
    pub date_utc: chrono::DateTime<chrono::Utc>,
    // egg_mode uses i32 for these two, might as well mimic it
    pub retweet_count: i32,
    pub favorite_count: i32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct TwitterAccount {
    pub handle: String, // Includes "@"
    pub category: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct QueryResult {
    pub query: String,
    pub date_utc: chrono::DateTime<chrono::Utc>,
    pub tweets: Vec<Tweet>,
}

/// Maximum for egg-mode
const N_TWEETS_PER_PAGE: u32 = 100;

/// account_handle includes the "@"
pub async fn search_for(token: &egg_mode::Token, query: String) {
    let response = egg_mode::search::search(query.clone())
        .result_type(egg_mode::search::ResultType::Recent)
        .count(N_TWEETS_PER_PAGE)
        .call(&token)
        .await
        .unwrap()
        .response;

    let query_result = QueryResult::new(query.as_str(), chrono::Utc::now(), &response);
    if StorageHandler::new().save_query(&query_result).is_err() {
        eprintln!("Could not store query!");
    }
}

/// Analyze multiple accounts as deserialized from configuration
pub async fn run_query_from_config(token: &egg_mode::Token, config: crate::twitter::Config) {
    // Map accounts to analyzation calls
    let futures: Vec<_> = config
        .accounts
        .into_iter()
        .map(|acc| search_for(&token, acc.handle))
        .collect();

    for f in futures {
        f.await;
    }
}

/// Parse an egg_mode::search::SearchResult into a serializable vector of tweets
pub fn search_to_tweet_vec(search: &egg_mode::search::SearchResult) -> Vec<Tweet> {
    let mut tweets = Vec::new();
    for tweet in &search.statuses {
        // TODO Clean this up, it's super weird
        let temp = &*(tweet.user.as_ref().unwrap());
        let handle = temp.screen_name.clone();
        tweets.push(Tweet {
            handle,
            text: tweet.text.to_owned(),
            date_utc: tweet.created_at,
            retweet_count: tweet.retweet_count,
            favorite_count: tweet.favorite_count,
        })
    }
    tweets
}

impl QueryResult {
    pub fn new(
        query: &str,
        date_utc: chrono::DateTime<chrono::Utc>,
        search: &egg_mode::search::SearchResult,
    ) -> QueryResult {
        QueryResult {
            query: query.to_string(),
            date_utc,
            tweets: search_to_tweet_vec(&search),
        }
    }

    pub fn deserialize(path: PathBuf) -> Result<QueryResult, io::Error> {
        Ok(serde_json::from_slice(&fs::read(path)?)?)
    }
}

// Not used, but can be useful for testing
fn _print_tweets(search_result: &egg_mode::search::SearchResult) {
    for tweet in &search_result.statuses {
        println!(
            "(@{}) {}",
            tweet.user.as_ref().unwrap().screen_name,
            tweet.text
        );
    }
}

pub mod auth {
    /// Reads token string from `token_path` and trims whitespace
    pub fn get_token(token_path: &std::path::Path) -> Option<egg_mode::Token> {
        let token_str = std::fs::read_to_string(token_path);
        if token_str.is_err() {
            eprintln!("Could not read {:?}", token_path);
            return None;
        }
        let token_str = token_str.unwrap().trim().to_string();
        Some(egg_mode::auth::Token::Bearer(token_str))
    }
}

#[tokio::test]
async fn test_authentication() {
    let maybe_token = auth::get_token(std::path::Path::new("auth/bearer.token"));
    assert!(maybe_token.is_some());
    let token = maybe_token.unwrap();
    let user = egg_mode::user::show("twitter", &token).await;
    assert!(user.is_ok());
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub categories: Vec<String>,
    pub accounts: Vec<TwitterAccount>,
}

impl Config {
    pub fn get(config_path: &std::path::Path) -> Option<Config> {
        let json_str = std::fs::read_to_string(config_path);
        if json_str.is_err() {
            eprintln!("Could not read {:?}", config_path);
            return None;
        }
        let maybe_json: serde_json::Result<Config> =
            serde_json::from_str(json_str.unwrap().as_str());
        if maybe_json.is_err() {
            eprintln!("serde_json parse error");
            return None;
        }

        Some(maybe_json.unwrap())
    }
}

#[tokio::test]
async fn test_json_parse() {
    let maybe_json = Config::get(&std::path::Path::new("conf/accounts.json"));
    assert!(maybe_json.is_some());
    let json = maybe_json.unwrap();
    let test_category = "news".to_string();
    assert!(json.categories.contains(&test_category))
}
