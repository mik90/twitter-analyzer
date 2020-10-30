# twitter-analyzer

## How-to
### Queries
- Run a query for `@twitter` with `cargo run --release -- query @twitter`
- Run query on accounts from `conf/accounts.json` with `cargo run --release query`
### Analysis
- Analyze all stored queries with `cargo run --release -- analyze`

## What it does
- Runs searches for the mentioned username(s) and then figures out
  - Most common words  
  - Most twitter handle pattern (e.g. CamelCase or namewithnumbers1234)

#### conf/
- accounts.json: List of accounts to search (if not supplied on command line)

#### auth/
- Twitter API keys and tokens. Use your own bearer token.

#### Storage
- Stores queries and analyses in json with serde_json
- Analyses are stored as `analyses/\<handle\>/\<search_date\>/analysis.json`
- Queries are stored as `queries/\<handle\>/\<search_date\>/query-result.json`


#### Todo
- Remove articles and other common words