mod analysis;
mod storage;
mod twitter;
mod util;

extern crate clap;
use analysis::{run_analysis_on_query, run_analysis_with_config, AnalysisConfig};
use clap::{App, Arg, SubCommand};
use std::path::Path;
use std::process::exit;
use twitter::*;

#[tokio::main]
async fn main() {
    let matches = App::new("twitter-analyzer")
        .version("0.1")
        .author("Mike Kaliman <kaliman.mike@gmail.com>")
        .about("Finds common words and handles in a twitter search")
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
                    Arg::with_name("bearer_token")
                        .short("t")
                        .long("bearer-token")
                        .value_name("TOKEN_PATH")
                        .help("File containing the bearer token, do not include newlines in it"),
                ),
        )
        .subcommand(
            SubCommand::with_name("clean")
                .about("Clean query/analysis storage directory before searching"),
        )
        .get_matches();

    match matches.subcommand() {
        ("analyze", Some(matches)) => {
            let query_to_analyze = matches.value_of("analyze_command");
            let storage_dir = Path::new(storage::DEFAULT_STORAGE_DIR);
            let config = AnalysisConfig::new(&std::path::Path::new("conf/analysis.json")).unwrap();
            let start = std::time::Instant::now();
            if query_to_analyze.is_some() {
                let query_to_analyze = query_to_analyze.unwrap();
                println!(
                    "Running analysis on queries for \"{}\"...",
                    query_to_analyze
                );
                let result = run_analysis_on_query(config, storage_dir, query_to_analyze).await;
                if result.is_err() {
                    eprintln!("Could not run analysis: {}", result.unwrap_err());
                    exit(1);
                }
            } else {
                println!("Running analysis on all available queries...");
                let result = run_analysis_with_config(config, storage_dir).await;
                if result.is_err() {
                    eprintln!("Could not run analysis: {}", result.unwrap_err());
                    exit(1);
                }
            }
            println!(
                "Time to analyze accounts from configuration: {} milliseconds",
                (std::time::Instant::now() - start).as_millis()
            )
        }
        ("clean", _) => {
            let res: Result<(), std::io::Error> =
                util::clear_directory(&Path::new(storage::DEFAULT_STORAGE_DIR));
            if res.is_err() {
                eprintln!("Error clearing out storage dir: {:?}", res.unwrap_err());
                exit(1)
            }
        }
        ("query", Some(matches)) => {
            let token_path = matches
                .value_of("bearer_token")
                .unwrap_or("auth/bearer.token");
            let maybe_token = auth::get_token(&std::path::Path::new(token_path));
            if maybe_token.is_none() {
                eprintln!("Could not get the bearer token!");
                exit(1)
            }

            let maybe_search_query = matches.value_of("search_query");
            if maybe_search_query.is_some() {
                // Search from command line arg
                let search_query = maybe_search_query.unwrap();
                println!("Searching for {:?}", &search_query);

                let start = std::time::Instant::now();
                search_for(&maybe_token.unwrap(), search_query.to_owned()).await;
                println!(
                    "Time to analyze {}: {} milliseconds",
                    search_query,
                    (std::time::Instant::now() - start).as_millis()
                )
            } else {
                // No command line search query provided, search from configuration
                let config = Config::get(&std::path::Path::new("conf/accounts.json"));
                if config.is_none() {
                    std::process::exit(1);
                }
                let start = std::time::Instant::now();
                run_query_from_config(maybe_token.unwrap(), config.unwrap()).await;
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
