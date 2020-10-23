use std::path::{Path, PathBuf};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Tweet {
  pub text: String,
  pub handle: String,
  pub date_utc: chrono::DateTime<chrono::Utc>,
  pub retweet_count: i32,
  pub favorite_count: i32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct TwitterAccount {
  pub handle: String, // Includes "@"
  pub category: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct QueryResult {
  pub query: String,
  pub date_utc: chrono::DateTime<chrono::Utc>,
  pub tweets: Vec<Tweet>,
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
  /// Saves to $PWD/<base_dir>/<handle>/<search-date>/query-result.json
  pub fn storage_location(&self, base_dir: &Path) -> PathBuf {
    // ISO 8601 / RFC 3339 date & time format
    let mut path = PathBuf::from(base_dir);
    path.push(&self.query);
    path.push(&self.date_utc.format("%+").to_string());
    path.push("query-result.json");
    path
  }
}

// Not used, but can be useful
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
  pub fn get_token(token_path: &std::path::Path) -> Option<egg_mode::Token> {
    let token_str = std::fs::read_to_string(token_path);
    if token_str.is_err() {
      eprintln!("Could not read {:?}", token_path);
      return None;
    }
    Some(egg_mode::auth::Token::Bearer(token_str.unwrap()))
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
    let maybe_json: serde_json::Result<Config> = serde_json::from_str(json_str.unwrap().as_str());
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
