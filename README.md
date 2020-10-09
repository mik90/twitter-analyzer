# twitter-analyzer

## How-to
- Analyze a twitter handle with `cargo run --release -- -a @twitter`
- Analyze accounts from conf/accounts.json with just `cargo run`


### What it does
- Runs searches for the mentioned username(s) and then figures out
  - Most common words  
  - Most twitter handle pattern (e.g. CamelCase or namewithnumbers1234)

#### conf/
- accounts.json
    -Stores accounts whose replies will be analyzed, they can also be the start of a search for networks of accounts.

#### auth/
- Twitter API keys and tokens. Use your own bearer token.


#### Todo
- Find what features are more common to one query vs another
- Log search queries to check cumulative data
- Separate analysis from printing
#### Storage
- Store in actual db or just in json?
- Json
  - ./analysis/<handle>/<search-date>/analysis.json
