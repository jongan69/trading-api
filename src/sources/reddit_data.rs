use roux::Subreddit;
use serde_json::{json, Value};
use std::collections::HashSet;
use regex::Regex;
use std::env;
pub async fn get_reddit_trending_stocks() -> Vec<String> {
    println!("\n🔍 Scraping Reddit for trending stocks...");

    let mut reddit_stocks: HashSet<String> = HashSet::new();

    // Load credentials from environment variables
    let client_id = env::var("REDDIT_CLIENT_ID").unwrap_or_default();
    let client_secret = env::var("REDDIT_CLIENT_SECRET").unwrap_or_default();
    // let user_agent = env::var("REDDIT_USER_AGENT").unwrap_or_else(|_| "rust-bot/0.1".to_string());

    if client_id.is_empty() || client_secret.is_empty() {
        println!("  Reddit credentials not found in environment variables. Skipping Reddit scraping.");
        return vec![];
    }

    // Define subreddits to scrape
    let subreddits = vec!["wallstreetbets", "stocks", "investing"];

    // Regex for stock tickers
    let ticker_re = Regex::new(r"\b[A-Z]{1,5}\b").unwrap();

    // Common words to ignore
    let ignore_words: HashSet<&'static str> = [
        "THE", "AND", "FOR", "YOU", "ARE", "WAS", "HAS", "HAD", "NOT", "BUT", "ALL", "CAN", "HER",
        "WERE", "SHE", "HIS", "ONE", "SAID", "THEY", "EACH", "WHICH", "DO", "HOW", "THEIR", "IF",
        "WILL", "UP", "OTHER", "ABOUT", "OUT", "MANY", "THEN", "THEM", "THESE", "SO", "SOME",
        "WOULD", "MAKE", "LIKE", "INTO", "HIM", "TIME", "TWO", "MORE", "GO", "NO", "WAY", "COULD",
        "MY", "THAN", "FIRST", "BEEN", "CALL", "WHO", "ITS", "NOW", "FIND", "LONG", "DOWN", "DAY",
        "DID", "GET", "COME", "MADE", "MAY", "PART"
    ]
    .iter()
    .cloned()
    .collect();

    // Loop through subreddits
    for subreddit_name in subreddits {
        println!("  Scraping r/{subreddit_name}...");

        let subreddit = Subreddit::new(subreddit_name);
        match subreddit.hot(20, None).await {
            Ok(listing) => {
                for post in listing.data.children {
                    let title = post.data.title.to_uppercase();
                    let text = post.data.selftext.to_uppercase();
                    let combined = format!("{title} {text}");

                    for cap in ticker_re.find_iter(&combined) {
                        let ticker = cap.as_str();
                        if !ignore_words.contains(ticker) {
                            reddit_stocks.insert(ticker.to_string());
                        }
                    }
                }
            }
            Err(e) => {
                println!("  Error scraping r/{subreddit_name}: {e}");
                continue;
            }
        }
    }

    reddit_stocks.into_iter().collect()
}

pub async fn get_reddit_news() -> Result<Value, String> {
    // Default subreddits and limit per subreddit
    let subreddits = vec!["wallstreetbets", "stocks", "investing"];
    let limit: usize = 25;

    let mut out = serde_json::Map::new();
    for name in subreddits {
        match get_subreddit_new_posts(name, limit).await {
            Ok(v) => {
                out.insert(name.to_string(), v);
            }
            Err(err) => {
                out.insert(
                    name.to_string(),
                    json!({ "error": err }),
                );
            }
        }
    }

    Ok(Value::Object(out))
}

/// Fetches the newest posts from a subreddit using roux's read-only API.
/// Returns a JSON array of simplified posts with key fields.
pub async fn get_subreddit_new_posts(subreddit_name: &str, limit: usize) -> Result<Value, String> {
    let subreddit = Subreddit::new(subreddit_name);
    let listing = subreddit
        .latest(limit as u32, None)
        .await
        .map_err(|e| format!("reddit new fetch error for r/{subreddit_name}: {e}"))?;

    let mut posts: Vec<Value> = Vec::new();
    for child in listing.data.children {
        let d = child.data;
        posts.push(json!({
            "id": d.id,
            "title": d.title,
            "author": d.author,
            "url": d.url,
            "permalink": d.permalink,
            "created_utc": d.created_utc,
            "num_comments": d.num_comments,
            "score": d.score,
            "selftext": d.selftext,
            "over_18": d.over_18,
            "stickied": d.stickied,
            "is_self": d.is_self,
            "subreddit": d.subreddit,
        }));
    }

    Ok(json!(posts))
}