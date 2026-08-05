pub async fn get_trending_penny_stocks() -> Vec<String> {
    let mut out = Vec::new();
    // yahoo
    let mut b = crate::sources::yahoo_data::get_trending_from_yahoo().await;
    out.append(&mut b);
    // reddit
    let mut c = crate::sources::reddit_data::get_reddit_trending_stocks().await;
    out.append(&mut c);
    let mut seen = std::collections::HashSet::new();
    out.retain(|s| seen.insert(s.clone()));
    out
}
