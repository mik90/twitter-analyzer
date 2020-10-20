mod analysis;
mod storage;
mod test;
mod twitter;

extern crate clap;
use analysis::{analyze_config, analyze_query};
use clap::{App, Arg, SubCommand};
use twitter::{auth, Config};

#[tokio::main]
async fn main() {
    // TODO Add config for token path and config path
    let matches = App::new("twitter-analyzer")
        .version("0.1")
        .author("Mike K. <kaliman.mike@gmail.com>")
        .about("Searches for analysis in twitter mentions")
        .subcommand(
            SubCommand::with_name("query")
                .about("Search twitter using a query and print analysis")
                .arg(
                    Arg::with_name("search_query")
                        .value_name("SEARCH_QUERY")
                        .help("Search query. Can include \"@\" if needed. Example: @twitter"),
                )
                .arg(
                    Arg::with_name("config")
                        .short("c")
                        .long("config")
                        .value_name("CONFIG_PATH")
                        .help("Path of json config with twitter handles"),
                )
                .arg(
                    Arg::with_name("bearer_token")
                        .short("t")
                        .long("bearer-token")
                        .value_name("TOKEN_PATH")
                        .help("File containing the bearer token, do not include newlines in it"),
                ),
        )
        .subcommand(
            SubCommand::with_name("clean").about("Clean analysis directory before searching"),
        )
        .get_matches();

    match matches.subcommand() {
        ("clean", _) => {
            std::fs::remove_dir_all(&storage::DEFAULT_STORAGE_LOCATION)
                .expect("Could not clean out storage area!");
        }
        ("query", Some(matches)) => {
            let token_path = matches
                .value_of("bearer_token")
                .unwrap_or("auth/bearer.token");
            let maybe_token = auth::get_token(&std::path::Path::new(token_path));
            if maybe_token.is_none() {
                eprintln!("Could not get the bearer token!");
                std::process::exit(1);
            }

            let maybe_search_query = matches.value_of("search_query");
            if maybe_search_query.is_some() {
                // Search from command line arg
                let search_query = maybe_search_query.unwrap();
                println!("Searching for {:?}", &search_query);

                let start = std::time::Instant::now();
                analyze_query(&maybe_token.unwrap(), search_query.to_owned()).await;
                println!(
                    "Time to analyze {}: {} milliseconds",
                    search_query,
                    (std::time::Instant::now() - start).as_millis()
                )
            } else {
                // No command line search query provided, search from configuration
                let config_path = matches.value_of("config").unwrap_or("conf/accounts.json");
                let maybe_config = Config::get(&std::path::Path::new(config_path));
                if maybe_config.is_none() {
                    std::process::exit(1);
                }
                println!("Analyzing queries from {}", config_path);
                let start = std::time::Instant::now();
                analyze_config(maybe_token.unwrap(), maybe_config.unwrap()).await;
                println!(
                    "Time to analyze accounts from configuration: {} milliseconds",
                    (std::time::Instant::now() - start).as_millis()
                )
            }
        }
        (_, _) => {
            eprintln!("Could not parse command line. Use \"--help\" to see available commands and subcommands");
            std::process::exit(1);
        }
    }
}
