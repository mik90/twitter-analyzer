#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct TwitterAccount {
  handle: String, // Includes "@"
  category: String,
}
use crate::patterns;
fn print_tweets(search_result: &egg_mode::search::SearchResult) {
  for tweet in &search_result.statuses {
    println!(
      "(@{}) {}",
      tweet.user.as_ref().unwrap().screen_name,
      tweet.text
    );
  }
}

/// account_handle includes the "@"
pub(crate) async fn analyze_account(token: &egg_mode::Token, account_handle: String) {
  let search_result = egg_mode::search::search(account_handle)
    .result_type(egg_mode::search::ResultType::Recent)
    .count(10)
    .call(&token)
    .await
    .unwrap();
  let words = patterns::get_most_common_words(&search_result);
  println!("Most common words: {:?}", words);
}

pub(crate) async fn analyze_accounts_from_config(token: egg_mode::Token, config: Config) {
  for account in config.accounts.into_iter() {
    analyze_account(&token, account.handle).await
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
