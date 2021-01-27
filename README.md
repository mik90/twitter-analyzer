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
- analysis.json: Configuration for discarding words
  - Prepositions grabbed from https://github.com/dariusk/corpora/blob/master/data/words/prepositions.json

#### auth/
- Twitter API keys and tokens. Use your own bearer token.

#### Storage
- Stores queries and analyses in json with serde_json
- Storage base directory defaults to `data/$QUERY`
  - Analyses are stored as `$SEARCH_DATE.analysis.json`
  - Queries are stored as `$SEARCH_DATE.query-result.json`


#### Todo
- Modularize storage so that a temp dir for tests isn't requried
  - Only serialize to disk if required
- Remove queries from most common words
- Make the retrieval configurable once instead of having to set the dir every time
- Make an analyzer or query struct? It could have a storage handler