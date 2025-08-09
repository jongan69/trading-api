use std::sync::Arc;

use axum::{extract::Query, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use futures::future::join_all;
use serde_json::{json, Value};
use time::{Date, OffsetDateTime};
use time::macros::format_description;

use crate::helpers::metrics;
use crate::helpers::options::black_scholes_delta;
use crate::helpers::params::{parse_symbols_csv, periods_per_year_from_interval};
use crate::services::yahoo::{fetch_prices_for_symbol, latest_close, metrics_for_prices};
use crate::sources;
use crate::state::AppState;
use crate::types::OptionsQuery;
use crate::errors::ApiError;

pub fn router(state: AppState) -> Router {
    Router::new().route("/options/recommendations", get(get_options_recommendations)).with_state(state)
}

#[utoipa::path(get, path = "/options/recommendations", params(OptionsQuery), tag = "options", responses((status = 200, description = "Rank options contracts")))]
pub async fn get_options_recommendations(axum::extract::State(state): axum::extract::State<AppState>, Query(q): Query<OptionsQuery>) -> Result<impl IntoResponse, ApiError> {
    let side = q.side.clone().unwrap_or_else(|| "both".to_string());
    let min_dte = q.min_dte.unwrap_or(7);
    let max_dte = q.max_dte.unwrap_or(60);
    let limit = q.limit.unwrap_or(20);
    let rf_annual = q.rf_annual.unwrap_or(0.03);
    let debug = q.debug.unwrap_or(true);

    let period_label = q.range.as_deref().unwrap_or("3mo");
    let interval = q.interval.as_deref().unwrap_or("1d");
    let periods_per_year = periods_per_year_from_interval(interval);

    let w_sharpe = q.sharpe_w.unwrap_or(0.4);
    let w_sortino = q.sortino_w.unwrap_or(0.4);
    let w_calmar = q.calmar_w.unwrap_or(0.2);

    let symbols_source = q.symbols_source.as_deref().unwrap_or("both").to_lowercase();
    let mut user_symbols: Vec<String> = vec![];
    if let Some(list) = q.symbols.as_ref() {
        user_symbols.extend(parse_symbols_csv(list));
    }
    if let Some(sym) = q.symbol.as_ref() {
        if !sym.is_empty() { user_symbols.push(sym.clone()); }
    }
    if debug {
        let uprev: Vec<&String> = user_symbols.iter().take(10).collect();
        println!("[options] symbols_source={}, user_symbols count={}, preview={:?}", symbols_source, user_symbols.len(), uprev);
    }

    let mut symbols: Vec<String> = vec![];
    if symbols_source == "yahoo" || symbols_source == "both" {
        if let Some(qstr) = q.yahoo_search.as_deref() {
            let ylimit = q.yahoo_limit.unwrap_or(25);
            if debug { println!("[options] yahoo search query='{}' limit={} ", qstr, ylimit); }
            match state.yahoo.search_ticker(qstr).await {
                Ok(resp) => {
                    let mut count = 0usize;
                    for item in resp.quotes.into_iter() {
                        let sym = item.symbol;
                        if !sym.trim().is_empty() {
                            symbols.push(sym);
                            count += 1;
                            if count >= ylimit { break; }
                        }
                    }
                    if debug { println!("[options] yahoo search added {} symbols", count); }
                }
                Err(err) => {
                    if debug { println!("[options] yahoo search error: {}", err); }
                }
            }
        }
        if q.yahoo_search.is_none() {
            let list = q.yahoo_list.as_deref().unwrap_or("most_actives");
            let ylimit = q.yahoo_limit.unwrap_or(25);
            let region = q.yahoo_region.as_deref().unwrap_or("US");
            if debug { println!("[options] yahoo list='{}' region='{}' limit={}", list, region, ylimit); }
            let fetched = if list.eq_ignore_ascii_case("trending") {
                sources::yahoo_data::yahoo_trending(region, ylimit).await
            } else {
                let scr_id = match list.to_ascii_lowercase().as_str() {
                    "gainers" => "day_gainers",
                    "losers" => "day_losers",
                    "most_actives" | "actives" => "most_actives",
                    _ => "most_actives",
                };
                sources::yahoo_data::yahoo_predefined_list(scr_id, ylimit).await
            };
            match fetched {
                Ok(mut syms) => {
                    if debug { println!("[options] yahoo predefined fetched {} symbols", syms.len()); }
                    symbols.append(&mut syms);
                }
                Err(err) => { if debug { println!("[options] yahoo predefined error: {}", err); } }
            }
        }
        if !user_symbols.is_empty() {
            let before = symbols.len();
            symbols.extend(user_symbols.clone());
            if debug { println!("[options] appended user-provided yahoo symbols: {} -> {}", before, symbols.len()); }
        }
        if symbols_source == "yahoo" && symbols.is_empty() {
            return Err(ApiError::BadRequest("symbols_source=yahoo requires yahoo_search or 'symbols'/'symbol' params".to_string()));
        }
        if debug { println!("[options] yahoo stage symbols count: {}", symbols.len()); }
    }

    if symbols_source == "finviz" || symbols_source == "both" {
        let signal = q.signal.as_deref().unwrap_or("TopGainers");
        let order = q.order.as_deref().unwrap_or("MarketCap");
        let screener = q.screener.as_deref().unwrap_or("Performance");
        let symbols_limit = q.symbols_limit.unwrap_or(20);
        if debug { println!("[options] finviz source: signal={}, order={}, screener={}, limit={}", signal, order, screener, symbols_limit); }
        match sources::finviz_data::fetch_finviz_symbols(signal, order, screener, symbols_limit).await {
            Ok(fetched) => {
                symbols.extend(fetched);
                if debug { println!("[options] finviz symbols count: {}", symbols.len()); }
            }
            Err(err) => {
                if debug { println!("[options] finviz error: {}", err); }
                return Err(ApiError::Upstream(err));
            }
        }
        if symbols_source == "finviz" {
            let before = symbols.len();
            symbols.extend(user_symbols);
            if debug { println!("[options] appended user-provided symbols (finviz mode): {} -> {}", before, symbols.len()); }
        }
    }
    {
        use std::collections::HashSet;
        let mut seen: HashSet<String> = HashSet::new();
        symbols.retain(|s| seen.insert(s.clone()));
    }
    if debug { println!("[options] symbols after dedup: {}", symbols.len()); }
    if symbols.is_empty() {
        if debug { println!("[options] no symbols after sourcing"); }
        return Err(ApiError::BadRequest("no symbols available".to_string()));
    }
    if debug {
        let preview: Vec<&String> = symbols.iter().take(10).collect();
        println!("[options] symbols ({}): {:?}{}", symbols.len(), preview, if symbols.len() > 10 { " ..." } else { "" });
    }

    if let Some(top_n) = q.underlying_top {
        if debug { println!("[options] underlying_top requested: {}", top_n); }
        let weights_outer = metrics::CompositeWeights { sharpe: w_sharpe, sortino: w_sortino, calmar: w_calmar };
        let yahoo_outer = state.yahoo.clone();
        let rank_futs = symbols.iter().cloned().map(move |sym| {
            let period_label = period_label.to_string();
            let weights = weights_outer.clone();
            let yahoo = yahoo_outer.clone();
            async move {
                let prices = match fetch_prices_for_symbol(&yahoo, &sym, &period_label).await { Ok(p) => p, Err(_) => return None };
                let m = metrics_for_prices(&prices, rf_annual, rf_annual, periods_per_year, Some(weights));
                Some((sym, m.composite_score))
            }
        });
        let mut scored: Vec<(String, f64)> = join_all(rank_futs).await.into_iter().flatten().collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        if debug {
            let prev: Vec<_> = scored.iter().take(5).map(|(s, sc)| format!("{}:{:.3}", s, sc)).collect();
            println!("[options] underlying ranking top5: {:?}", prev);
        }
        if scored.len() > top_n { scored.truncate(top_n); }
        symbols = scored.into_iter().map(|(s, _)| s).collect();
        if debug { println!("[options] symbols after underlying_top {}", symbols.len()); }
    }

    let min_premium = q.min_premium;
    let max_premium = q.max_premium;
    let min_volume = q.min_volume;
    let min_delta = q.min_delta;
    let max_delta = q.max_delta;
    let min_sr = q.min_strike_ratio;
    let max_sr = q.max_strike_ratio;
    let per_symbol_limit = q.per_symbol_limit.unwrap_or(usize::MAX);
    let max_spread_pct = q.max_spread_pct;

    let q_arc = Arc::new(q.clone());
    let tasks = symbols.into_iter().map(|symbol| {
        let side = side.clone();
        let q_local = q_arc.clone();
        let debug_local = debug;
        let yahoo = state.yahoo.clone();
        async move {
            if debug_local { println!("[options][{}] fetch spot & prices", symbol); }
            let spot = match latest_close(&yahoo, &symbol).await { Ok(s) => s, Err(e) => { if debug_local { println!("[options][{}] spot error: {}", symbol, e); } return Vec::new() } };
            let prices = match fetch_prices_for_symbol(&yahoo, &symbol, period_label).await { Ok(p) => p, Err(e) => { if debug_local { println!("[options][{}] prices error: {}", symbol, e); } return Vec::new() } };
            let returns = metrics::compute_returns_from_prices(&prices);
            if debug_local { println!("[options][{}] spot={}, returns_len={}", symbol, spot, returns.len()); }
            let under_metrics = metrics_for_prices(&prices, rf_annual, rf_annual, periods_per_year, Some(metrics::CompositeWeights { sharpe: w_sharpe, sortino: w_sortino, calmar: w_calmar }));
            let base_score = under_metrics.composite_score;
            if debug_local { println!("[options][{}] composite={:.4}", symbol, base_score); }

            let now_ts = OffsetDateTime::now_utc().unix_timestamp();
            let mut out: Vec<Value> = Vec::new();
            if debug_local { println!("[options][{}] fetch alpaca snapshots", symbol); }
            if let Ok(v) = sources::alpaca_data::fetch_alpaca_snapshots(&symbol, &q_local).await {
                if let Some(snaps) = v.get("snapshots").and_then(|s| s.as_array()) {
                    if debug_local { println!("[options][{}] snapshots: {}", symbol, snaps.len()); }
                    let fmt = format_description!("[year]-[month]-[day]");
                    for s in snaps {
                        let contract_symbol = s.get("symbol").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let details = s.get("details");
                        let strike = details.and_then(|d| d.get("strike_price")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let exp_ts = if let Some(exp) = details.and_then(|d| d.get("expiration_date")).and_then(|v| v.as_str()) {
                            if let Ok(date) = Date::parse(exp, &fmt) { date.with_hms(0,0,0).unwrap().assume_utc().unix_timestamp() } else { 0 }
                        } else { 0 };
                        let typ = details.and_then(|d| d.get("type")).and_then(|v| v.as_str()).unwrap_or("");
                        let is_call = typ.eq_ignore_ascii_case("call") || contract_symbol.ends_with('C');
                        let is_put = typ.eq_ignore_ascii_case("put") || contract_symbol.ends_with('P');
                        if side == "call" && !is_call { continue; }
                        if side == "put" && !is_put { continue; }
                        if strike <= 0.0 || exp_ts <= 0 { continue; }
                        let quote = s.get("latest_quote");
                        let trade = s.get("latest_trade");
                        let bid = quote.and_then(|q| q.get("bid_price")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let ask = quote.and_then(|q| q.get("ask_price")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let last = trade.and_then(|t| t.get("price")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let volume = trade.and_then(|t| t.get("size")).and_then(|v| v.as_u64()).unwrap_or(0);
                        let greeks = s.get("greeks");
                        let iv = greeks.and_then(|g| g.get("iv")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let delta_from_feed = greeks.and_then(|g| g.get("delta")).and_then(|v| v.as_f64());
                        let dte_days = ((exp_ts - now_ts) as f64 / 86_400.0).max(0.0);
                        if dte_days < min_dte as f64 || dte_days > max_dte as f64 { continue; }
                        let premium = if bid > 0.0 && ask > 0.0 { (bid + ask) / 2.0 } else { last };
                        if premium <= 0.0 { continue; }
                        if let Some(min_v) = min_volume { if volume < min_v { continue; } }
                        if let Some(min_p) = min_premium { if premium < min_p { continue; } }
                        if let Some(max_p) = max_premium { if premium > max_p { continue; } }
                        let t_years = dte_days / 365.0;
                        let delta = delta_from_feed.unwrap_or_else(|| black_scholes_delta(spot, strike, rf_annual, iv.abs(), t_years, is_call).unwrap_or(0.0));
                        if let Some(min_d) = min_delta { if delta < min_d { continue; } }
                        if let Some(max_d) = max_delta { if delta > max_d { continue; } }
                        let strike_ratio = strike / spot;
                        if let Some(lo) = min_sr { if strike_ratio < lo { continue; } }
                        if let Some(hi) = max_sr { if strike_ratio > hi { continue; } }
                        let mid = if bid > 0.0 && ask > 0.0 { (bid + ask) / 2.0 } else { premium };
                        let spread = if ask > 0.0 && bid > 0.0 { ask - bid } else { 0.0 };
                        let spread_pct = if mid > 0.0 { spread / mid } else { f64::INFINITY };
                        if let Some(max_sp) = max_spread_pct { if spread_pct.is_finite() && spread_pct > max_sp { continue; } }
                        let leverage = (delta.abs() * spot) / premium;
                        let score = base_score * delta * (spot / premium) / (1.0 + dte_days / 30.0);
                        out.push(json!({
                            "symbol": symbol,
                            "contract": contract_symbol,
                            "side": if is_call { "call" } else { "put" },
                            "strike": strike,
                            "expiration": exp_ts,
                            "dte_days": dte_days,
                            "premium": premium,
                            "mid": mid,
                            "spread": spread,
                            "spread_pct": spread_pct,
                            "implied_vol": iv,
                            "delta": delta,
                            "leverage": leverage,
                            "volume": volume,
                            "open_interest": 0u64,
                            "strike_ratio": strike_ratio,
                            "score": score,
                            "underlying_metrics": under_metrics,
                        }));
                    }
                }
            } else {
                if debug_local { println!("[options][{}] falling back to yahoo options", symbol); }
                if let Ok(chain) = sources::yahoo_data::fetch_yahoo_options_chain(&symbol).await {
                    if let Some(result) = chain.get("optionChain").and_then(|c| c.get("result")).and_then(|r| r.as_array()).and_then(|a| a.get(0)) {
                        let now_ts = OffsetDateTime::now_utc().unix_timestamp();
                        let options = result.get("options").and_then(|o| o.as_array()).and_then(|a| a.get(0));
                        let process = |arr: Option<&Value>, is_call: bool, out: &mut Vec<Value>| {
                            if let Some(arr) = arr.and_then(|v| v.as_array()) {
                                for c in arr {
                                    let contract_symbol = c.get("contractSymbol").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                    let strike = c.get("strike").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                    let exp_ts = c.get("expiration").and_then(|v| v.as_i64()).unwrap_or(0);
                                    let bid = c.get("bid").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                    let ask = c.get("ask").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                    let last = c.get("lastPrice").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                    let volume = c.get("volume").and_then(|v| v.as_u64()).unwrap_or(0);
                                    if strike <= 0.0 || exp_ts <= 0 { continue; }
                                    let dte_days = ((exp_ts - now_ts) as f64 / 86_400.0).max(0.0);
                                    if dte_days < min_dte as f64 || dte_days > max_dte as f64 { continue; }
                                    let premium = if bid > 0.0 && ask > 0.0 { (bid + ask) / 2.0 } else { last };
                                    if premium <= 0.0 { continue; }
                                    if let Some(min_v) = min_volume { if volume < min_v { continue; } }
                                    if let Some(min_p) = min_premium { if premium < min_p { continue; } }
                                    if let Some(max_p) = max_premium { if premium > max_p { continue; } }
                                    let t_years = dte_days / 365.0;
                                    let iv = c.get("impliedVolatility").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                    let delta_from_feed = c.get("delta").and_then(|v| v.as_f64());
                                    let delta = delta_from_feed.unwrap_or_else(|| black_scholes_delta(spot, strike, rf_annual, iv.abs(), t_years, is_call).unwrap_or(0.0));
                                    if let Some(min_d) = min_delta { if delta < min_d { continue; } }
                                    if let Some(max_d) = max_delta { if delta > max_d { continue; } }
                                    let strike_ratio = strike / spot;
                                    if let Some(lo) = min_sr { if strike_ratio < lo { continue; } }
                                    if let Some(hi) = max_sr { if strike_ratio > hi { continue; } }
                                    let mid = if bid > 0.0 && ask > 0.0 { (bid + ask) / 2.0 } else { premium };
                                    let spread = if ask > 0.0 && bid > 0.0 { ask - bid } else { 0.0 };
                                    let spread_pct = if mid > 0.0 { spread / mid } else { f64::INFINITY };
                                    if let Some(max_sp) = max_spread_pct { if spread_pct.is_finite() && spread_pct > max_sp { continue; } }
                                    let leverage = (delta.abs() * spot) / premium;
                                    let score = base_score * delta * (spot / premium) / (1.0 + dte_days / 30.0);
                                    out.push(json!({
                                        "symbol": symbol,
                                        "contract": contract_symbol,
                                        "side": if is_call { "call" } else { "put" },
                                        "strike": strike,
                                        "expiration": exp_ts,
                                        "dte_days": dte_days,
                                        "premium": premium,
                                        "mid": mid,
                                        "spread": spread,
                                        "spread_pct": spread_pct,
                                        "implied_vol": iv,
                                        "delta": delta,
                                        "leverage": leverage,
                                        "volume": volume,
                                        "open_interest": 0u64,
                                        "strike_ratio": strike_ratio,
                                        "score": score,
                                        "underlying_metrics": under_metrics,
                                    }));
                                }
                            }
                        };
                        if let Some(opts) = options {
                            process(opts.get("calls"), true, &mut out);
                            process(opts.get("puts"), false, &mut out);
                        }
                    }
                }
            }
            out.sort_by(|a, b| {
                let sa = a.get("score").and_then(|v| v.as_f64()).unwrap_or(f64::MIN);
                let sb = b.get("score").and_then(|v| v.as_f64()).unwrap_or(f64::MIN);
                sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
            });
            if out.len() > per_symbol_limit { out.truncate(per_symbol_limit); }
            if debug_local { println!("[options][{}] after per_symbol_limit => {}", symbol, out.len()); }
            out
        }
    });

    let mut options_list: Vec<Value> = join_all(tasks).await.into_iter().flatten().collect();
    options_list.sort_by(|a, b| {
        let sa = a.get("score").and_then(|v| v.as_f64()).unwrap_or(f64::MIN);
        let sb = b.get("score").and_then(|v| v.as_f64()).unwrap_or(f64::MIN);
        sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
    });
    options_list.truncate(limit);
    if debug { println!("[options] total selected: {} (limit {})", options_list.len(), limit); }

    Ok((StatusCode::OK, Json(json!({ "results": options_list }))))
}


