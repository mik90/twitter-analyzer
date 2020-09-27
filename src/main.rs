/* - Look at 10 most recent tweets
 * - Look at max 10 most recent replies of those tweets
 *      - The API doesn't directly allow for this:
 *          - https://stackoverflow.com/questions/29928638/getting-tweet-replies-to-a-particular-tweet-from-a-particular-user
 * - Just searching for mentions instead
 * - For each search result, spawn a new Tokio task that handles it
 * - All of the data should be put somewhere
 *      - sent to another thread that combines the data together?
 */

use serde::{Deserialize, Serialize};
use serde_json::Result;

mod patterns;

#[derive(Serialize, Deserialize)]
pub struct TwitterAccount {
    handle: String, // Includes "@"
    category: String,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    categories: Vec<String>,
    accounts: Vec<TwitterAccount>,
}

pub fn parse_config_json() -> Option<Config> {
    let json_str = std::fs::read_to_string("conf/accounts.json");
    if json_str.is_err() {
        eprintln!("Could not parse conf/accounts.json");
        return None;
    }
    let maybe_json: Result<Config> = serde_json::from_str(json_str.unwrap().as_str());
    if maybe_json.is_err() {
        eprintln!("serde_json parse error");
        return None;
    }

    Some(maybe_json.unwrap())
}

pub fn get_token() -> Option<egg_mode::Token> {
    let token_str = std::fs::read_to_string("auth/bearer.token");
    if token_str.is_err() {
        eprintln!("Could not parse auth/bearer.token");
        return None;
    }
    Some(egg_mode::auth::Token::Bearer(token_str.unwrap()))
}

async fn analyze_accounts(token: egg_mode::Token, config: Config) {
    for account in config.accounts.into_iter() {
        let search = egg_mode::search::search(account.handle)
            .result_type(egg_mode::search::ResultType::Recent)
            .count(10)
            .call(&token)
            .await
            .unwrap();

        // TODO Spawn task that handles tweets for the account
        for tweet in &search.statuses {
            println!(
                "(@{}) {}",
                tweet.user.as_ref().unwrap().screen_name,
                tweet.text
            );
        }
    }
}

#[tokio::main]
async fn main() {
    let maybe_token = get_token();
    if maybe_token.is_none() {
        std::process::exit(1);
    }
    let maybe_config = parse_config_json();
    if maybe_config.is_none() {
        std::process::exit(1);
    }

    let config = maybe_config.unwrap();
    let token = maybe_token.unwrap();
    analyze_accounts(token, config).await;
}

#[tokio::test]
async fn test_authentication() {
    let maybe_token = get_token();
    assert!(maybe_token.is_some());
    let token = maybe_token.unwrap();
    let user = egg_mode::user::show("twitter", &token).await;
    assert!(user.is_ok());
}

#[tokio::test]
async fn test_json_parse() {
    let maybe_json = parse_config_json();
    assert!(maybe_json.is_some());
    let json = maybe_json.unwrap();
    let test_category = "news".to_string();
    assert!(json.categories.contains(&test_category))
}
