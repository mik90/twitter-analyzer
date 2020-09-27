mod patterns;
mod twitter;

extern crate clap;
use clap::{App, Arg};
use twitter::{analyze_account, analyze_accounts_from_config, auth, Config};

#[tokio::main]
async fn main() {
    // TODO Add config for token path and config path
    let matches = App::new("twitter-bot")
        .version("0.1")
        .author("Mike K. <kaliman.mike@gmail.com>")
        .about("Searches for patterns in twitter mentions")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("CONFIG_PATH")
                .help("Path of json config with twitter handles"),
        )
        .arg(
            Arg::with_name("account")
                .short("a")
                .long("account")
                .value_name("TWITTER_HANDLE")
                .help("Account to search (including handle). Example: @twitter"),
        )
        .arg(
            Arg::with_name("bearer_token")
                .short("t")
                .long("bearer_token")
                .value_name("TOKEN_PATH")
                .help("File containing the bearer token, do not include newlines in it"),
        )
        .get_matches();

    let token_path = matches
        .value_of("bearer_token")
        .unwrap_or("auth/bearer.token");
    let maybe_token = auth::get_token(&std::path::Path::new(token_path));
    if maybe_token.is_none() {
        std::process::exit(1);
    }

    if matches.is_present("account") {
        analyze_account(
            &maybe_token.unwrap(),
            matches.value_of("account").unwrap().to_string(),
        )
        .await;
    } else {
        let config_path = matches.value_of("config").unwrap_or("conf/accounts.json");
        let maybe_config = Config::get(&std::path::Path::new(config_path));
        if maybe_config.is_none() {
            std::process::exit(1);
        }
        analyze_accounts_from_config(maybe_token.unwrap(), maybe_config.unwrap()).await;
    }
}
