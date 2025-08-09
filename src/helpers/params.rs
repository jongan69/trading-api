pub fn periods_per_year_from_interval(interval: &str) -> usize {
    match interval {
        "1wk" => 52,
        "1mo" => 12,
        _ => 252,
    }
}

pub fn parse_symbols_csv(s: &str) -> Vec<String> {
    s.split(',')
        .filter(|t| !t.trim().is_empty())
        .map(|t| t.trim().to_string())
        .collect()
}


