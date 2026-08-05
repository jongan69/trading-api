// ── API Client ──────────────────────────────────────────────
const STORAGE_KEY = 'trading_api_base_url';

export function getBaseUrl() {
  const params = new URLSearchParams(window.location.search);
  return params.get('api') || localStorage.getItem(STORAGE_KEY) || 'http://localhost:3000';
}

export function setBaseUrl(url) {
  localStorage.setItem(STORAGE_KEY, url);
}

let baseUrl = getBaseUrl();

async function apiFetch(path, params = {}) {
  const url = new URL(path, baseUrl);
  Object.entries(params).forEach(([k, v]) => {
    if (v !== undefined && v !== null && v !== '') url.searchParams.set(k, v);
  });

  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), 10000);

  try {
    const res = await fetch(url.toString(), { signal: controller.signal });
    clearTimeout(timer);
    if (!res.ok) {
      const body = await res.json().catch(() => ({}));
      throw new Error(body.error || `HTTP ${res.status}`);
    }
    return await res.json();
  } catch (err) {
    clearTimeout(timer);
    if (err.name === 'AbortError') throw new Error('Request timed out');
    throw err;
  }
}

// ── Health ───────────────────────────────────────────────────
export async function getHealth() {
  return apiFetch('/health');
}

// ── Market Overview ──────────────────────────────────────────
export async function getMarketOverview() {
  return apiFetch('/coingecko/market-overview');
}

export async function getTopGainers(limit = 5) {
  return apiFetch('/coingecko/gainers', { limit });
}

// ── Crypto ───────────────────────────────────────────────────
export async function getTopCoins(limit = 50) {
  return apiFetch('/coingecko/top', { limit });
}

export async function getTrendingCrypto(limit = 10) {
  return apiFetch('/trending/crypto', { limit });
}

export async function getKrakenTicker(pairs = '') {
  return apiFetch('/kraken/ticker', pairs ? { pairs } : {});
}

export async function getKrakenTrending(limit = 10) {
  return apiFetch('/kraken/trending', { limit });
}

export async function getSimplePrice(ids, vsCurrencies = 'usd') {
  return apiFetch('/coingecko/simple-price', { ids, vs_currencies: vsCurrencies });
}

// ── Stocks ───────────────────────────────────────────────────
export async function getTrendingStocks(limit = 25) {
  return apiFetch('/trending/stocks', { limit });
}

export async function getGroup(limit = 25) {
  return apiFetch('/group', { limit });
}

export async function getYahooRank(symbols, range = '1mo', interval = '1d') {
  return apiFetch('/rank/yahoo', { symbols, range, interval });
}

// ── Options ──────────────────────────────────────────────────
export async function getOptionsRecommendations(params = {}) {
  return apiFetch('/options/recommendations', {
    symbols: 'AAPL,MSFT,GOOGL,AMZN,NVDA,TSLA,META',
    side: 'call',
    min_dte: 7,
    max_dte: 60,
    limit: 30,
    range: '1mo',
    interval: '1d',
    ...params,
  });
}

export async function getTrendingOptions(limit = 10) {
  return apiFetch('/trending-options', { limit });
}

// ── News ─────────────────────────────────────────────────────
export async function getNews() {
  return apiFetch('/news');
}

// ── Forex ────────────────────────────────────────────────────
export async function getForex(limit = 25) {
  return apiFetch('/forex', { limit });
}

// ── Utility: normalize different response envelopes ──────────

/** Extract data array or object from whichever envelope the endpoint uses. */
export function unwrapData(response, key = null) {
  if (!response) return null;
  // Generic envelopes
  if (response.data !== undefined) return response.data;
  if (response.results !== undefined) return response.results;
  if (response.symbols !== undefined) return response.symbols;
  if (key && response[key] !== undefined) return response[key];
  return response;
}

/** Format a number with appropriate precision. */
export function fmt(n, decimals = 2) {
  if (n == null || isNaN(n)) return '—';
  const abs = Math.abs(n);
  if (abs >= 1e12) return (n / 1e12).toFixed(2) + 'T';
  if (abs >= 1e9)  return (n / 1e9).toFixed(2) + 'B';
  if (abs >= 1e6)  return (n / 1e6).toFixed(2) + 'M';
  if (abs >= 1e3)  return (n / 1e3).toFixed(2) + 'K';
  if (abs >= 1)    return n.toFixed(decimals);
  if (abs >= 0.01) return n.toFixed(decimals > 2 ? decimals : 4);
  return n.toFixed(decimals > 2 ? decimals : 6);
}

export function fmtPct(n) {
  if (n == null || isNaN(n)) return '—';
  return n.toFixed(2) + '%';
}

export function pctClass(n) {
  if (n == null || isNaN(n)) return '';
  return n >= 0 ? 'up' : 'down';
}
