mod analysis;
mod storage;
mod twitter;

extern crate clap;
use analysis::run_analysis;
use clap::{App, Arg, SubCommand};
use std::path::Path;
use storage::DEFAULT_QUERY_RESULT_DIR;
use twitter::*;

#[tokio::main]
async fn main() {
    // TODO Add config for token path and config path
    let matches = App::new("twitter-analyzer")
        .version("0.1")
        .author("Mike K. <kaliman.mike@gmail.com>")
        .about("Searches for analysis in twitter mentions")
        .subcommand(
            SubCommand::with_name("analyze")
                .about("Search twitter using a query and print analysis")
                .arg(
                    Arg::with_name("analyze_command")
                        .value_name("ANALYZE_COMMAND")
                        .help("Command for analyzing twitter queries"),
                ),
        )
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
            SubCommand::with_name("clean")
                .about("Clean analysis directory before searching")
                .arg(
                    Arg::with_name("queries")
                        .short("q")
                        .long("queries")
                        .help("Delete all queries"),
                )
                .arg(
                    Arg::with_name("analyses")
                        .short("a")
                        .long("analyses")
                        .help("Delete all analyses"),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        ("clean", Some(matches)) => {
            let clean_queries = matches.value_of("queries").is_some();
            let clean_analyses = matches.value_of("analyses").is_some();
            // Clean both if both args are present, or if none are
            let no_args_provided = !clean_queries && !clean_analyses;

            if (clean_queries || no_args_provided)
                && Path::new(&storage::DEFAULT_QUERY_RESULT_DIR).exists()
            {
                std::fs::remove_dir_all(&DEFAULT_QUERY_RESULT_DIR)
                    .expect("Could not clean out query storage area!");
            }

            if (clean_analyses || no_args_provided)
                && Path::new(&storage::DEFAULT_ANALYSIS_DIR).exists()
            {
                std::fs::remove_dir_all(&storage::DEFAULT_ANALYSIS_DIR)
                    .expect("Could not clean out analysis storage area!");
            }
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
                run_query(&maybe_token.unwrap(), search_query.to_owned()).await;
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
                run_query_from_config(maybe_token.unwrap(), maybe_config.unwrap()).await;
                println!(
                    "Time to analyze accounts from configuration: {} milliseconds",
                    (std::time::Instant::now() - start).as_millis()
                )
            }
        }
        ("analyze", Some(matches)) => {
            let maybe_command = matches.value_of("analyze_command");
            if maybe_command.is_some() {
                eprintln!("Specific commands not implemented yet!");
                std::process::exit(1);
            } else {
                // Run analysis on available queries
                println!("Running analysis on all available queries...");
                let result = run_analysis(Vec::new()).await;
                if result.is_err() {
                    eprintln!("Could not run analysis: {}", result.unwrap_err());
                    std::process::exit(1);
                }
            }
        }
        (_, _) => {
            eprintln!("Could not parse command line. Use \"--help\" to see available commands and subcommands");
            std::process::exit(1);
        }
    }
}
