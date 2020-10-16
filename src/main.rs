mod analysis;
mod storage;
mod test;
mod twitter;

extern crate clap;
use analysis::{analyze_account, analyze_config};
use clap::{App, Arg};
use twitter::{auth, Config};

#[tokio::main]
async fn main() {
    // TODO Add config for token path and config path
    let matches = App::new("twitter-bot")
        .version("0.1")
        .author("Mike K. <kaliman.mike@gmail.com>")
        .about("Searches for analysis in twitter mentions")
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
        .arg(
            Arg::with_name("clean")
                .long("clean")
                .help("Clean analysis directory before searching"),
        )
        .get_matches();

    let token_path = matches
        .value_of("bearer_token")
        .unwrap_or("auth/bearer.token");
    let maybe_token = auth::get_token(&std::path::Path::new(token_path));
    if maybe_token.is_none() {
        std::process::exit(1);
    }

    if matches.is_present("clean") {
        std::fs::remove_dir_all(&storage::DEFAULT_STORAGE_LOCATION)
            .expect("Could not clean out storage area!");
    }
    if matches.is_present("account") {
        let account_handle = matches.value_of("account").unwrap();

        let start = std::time::Instant::now();
        analyze_account(&maybe_token.unwrap(), account_handle.to_owned()).await;
        let end = std::time::Instant::now();
        let duration = end - start;
        println!(
            "Time to analyze {}: {} milliseconds",
            account_handle,
            duration.as_millis()
        )
    } else {
        let config_path = matches.value_of("config").unwrap_or("conf/accounts.json");
        let maybe_config = Config::get(&std::path::Path::new(config_path));
        if maybe_config.is_none() {
            std::process::exit(1);
        }

        let start = std::time::Instant::now();
        analyze_config(maybe_token.unwrap(), maybe_config.unwrap()).await;
        let end = std::time::Instant::now();
        let duration = end - start;
        println!(
            "Time to analyze accounts from {}: {} milliseconds",
            config_path,
            duration.as_millis()
        )
    }
}
