# twitter-analyzer


## What it does
- Runs searches for the mentioned username(s) and then figures out
  - Most common words  
  - Most twitter handle pattern (e.g. CamelCase or namewithnumbers1234)

## Usage
### Queries
- Run a query for `@twitter` with `cargo run --release -- query @twitter`
- Run query on all accounts from `conf/accounts.json` with `cargo run --release query`

### Analysis
- Analyze all stored queries with `cargo run --release -- analyze`
- Analyze single query for `@twitter` with `cargo run --release -- analyze @twitter`

#### conf/
- accounts.json: List of accounts to search (if not supplied on command line)
- analysis.json: Configuration for discarding words
  - Prepositions grabbed from https://github.com/dariusk/corpora/blob/master/data/words/prepositions.json

#### auth/
- Twitter API keys and tokens. Store your own bearer token under there in `auth/bearer.token`. Whitespace is trimmed.

#### Storage
- Stores queries and analyses in json with serde_json
- Storage base directory defaults to `data/$QUERY`
  - Analyses are stored as `$SEARCH_DATE.analysis.json`
  - Queries are stored as `$SEARCH_DATE.query-result.json`


#### Todo
- Remove queries from most common words
- Make an analyzer or query struct? It could have a storage handler
  - Storage dir could be configurable once instead of having to set the dir every time
- Move all logic in `main` to a function that returns a `Result<>