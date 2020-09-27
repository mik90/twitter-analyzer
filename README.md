# twitter-bot

## How-to
- Analyze twitter's account with `cargo run -- -a @twitter`
- Analyze accounts from conf/accounts.json with `cargo run`

## conf
### accounts.json
Stores accounts whose replies will be analyzed, they can also be the start of a search for
networks of accounts.

### auth
Twitter API keys and tokens

#### TODO
- Look at 10 most recent tweets
- Look at max 10 most recent replies of those tweets
  - The API doesn't directly allow for this:
    - https://stackoverflow.com/questions/29928638/getting-tweet-replies-to-a-particular-tweet-from-a-particular-user
- Just searching for mentions instead
- For each search result, spawn a new Tokio task that handles it
- All of the data should be put somewhere
  - sent to another thread that combines the data together?