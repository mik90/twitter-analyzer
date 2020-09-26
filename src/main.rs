// TODO Function that reads in keys/tokens
// bearer token might do, don't forget to strip the newlines

// TODO function that looks through accounts.json
use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Serialize, Deserialize)]
struct TwitterAccount {
    handle: String, // Includes "@"
    category: String,
}

#[derive(Serialize, Deserialize)]
struct Config {
    categories: Vec<String>,
    accounts: Vec<TwitterAccount>,
}

fn parse_config_json() -> Option<Config> {
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

fn get_token() -> Option<egg_mode::Token> {
    let token_str = std::fs::read_to_string("auth/bearer.token");
    if token_str.is_err() {
        eprintln!("Could not parse auth/bearer.token");
        return None;
    }
    Some(egg_mode::auth::Token::Bearer(token_str.unwrap()))
}

#[tokio::main]
async fn main() {
    let maybe_token = get_token();
    if maybe_token.is_none() {
        eprintln!("Could not get token!");
        std::process::exit(1);
    }
    let token = maybe_token.unwrap();
    let user = egg_mode::user::show("twitter", &token).await.unwrap();
    println!(
        "Found user with name {} and {} followers",
        user.name, user.followers_count
    );
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
