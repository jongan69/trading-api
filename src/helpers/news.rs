use crate::sources::alpaca_data::get_alpaca_news;
use crate::sources::finviz_data::fetch_finviz_news;
use crate::sources::reddit_data::get_reddit_news;
use serde_json::{json, Value};

pub async fn get_news() -> Result<Value, String> {
    // Get Finviz News (no-arg lib function)
    let finviz_news = fetch_finviz_news(None).await.unwrap_or(Value::Null);
    // Get Reddit News
    let reddit_news = get_reddit_news().await.unwrap_or(Value::Null);
    // Get Alpaca News
    let alpaca_news = get_alpaca_news().await.unwrap_or(Value::Null);
    Ok(json!({ "finviz": finviz_news, "reddit": reddit_news, "alpaca": alpaca_news }))
}
