// ── Dashboard Orchestrator ──────────────────────────────────
import * as API from './api.js';
import './lib/chart.js'; // auto-renders sparklines on DOM changes
import { renderMarketOverview } from './panels/market.js';
import { renderCrypto } from './panels/crypto.js';
import { renderStocks } from './panels/stocks.js';
import { renderOptions } from './panels/options.js';
import { renderNews } from './panels/news.js';
import { renderForex } from './panels/forex.js';

const panels = {
  crypto:  { render: renderCrypto,  interval: 30_000 },
  stocks:  { render: renderStocks,  interval: 60_000 },
  options: { render: renderOptions, interval: 60_000 },
  news:    { render: renderNews,    interval: 60_000 },
  forex:   { render: renderForex,   interval: 30_000 },
};

let currentTab = 'crypto';
let connected = false;
let pollTimers = {};

// ── Init ─────────────────────────────────────────────────────
async function init() {
  // Configurable API base
  const params = new URLSearchParams(window.location.search);
  if (params.get('api')) {
    API.setBaseUrl(params.get('api'));
    document.getElementById('api-label').textContent = params.get('api');
  } else {
    document.getElementById('api-label').textContent = API.getBaseUrl();
  }

  // Clock
  setInterval(() => {
    document.getElementById('clock').textContent = new Date().toLocaleTimeString();
  }, 1000);

  // Tab switching
  document.querySelectorAll('#tabbar button').forEach(btn => {
    btn.addEventListener('click', () => switchTab(btn.dataset.tab));
  });

  // Watchlist
  setupWatchlist();

  // Keyboard shortcuts
  document.addEventListener('keydown', e => {
    if (e.target.tagName === 'INPUT') return;
    const keys = { '1':'crypto','2':'stocks','3':'options','4':'news','5':'forex' };
    if (keys[e.key]) switchTab(keys[e.key]);
    if (e.key === 'r' || e.key === 'R') refreshCurrent();
  });

  // Check connectivity
  await checkHealth();
  setInterval(checkHealth, 15_000);

  // Start current tab
  switchTab(currentTab);
}

// ── Health Check ─────────────────────────────────────────────
async function checkHealth() {
  const led = document.getElementById('status-led');
  const text = document.getElementById('status-text');
  const disc = document.getElementById('disconnected');
  try {
    await API.getHealth();
    led.className = 'status-dot connected';
    text.textContent = 'connected';
    disc.classList.remove('show');
    connected = true;
  } catch {
    led.className = 'status-dot disconnected';
    text.textContent = 'disconnected';
    disc.classList.add('show');
    connected = false;
  }
}

// ── Tab Switching ────────────────────────────────────────────
function switchTab(tab) {
  currentTab = tab;
  document.querySelectorAll('#tabbar button').forEach(b => b.classList.remove('active'));
  document.querySelector(`#tabbar button[data-tab="${tab}"]`).classList.add('active');
  refreshCurrent();
}

// ── Polling ──────────────────────────────────────────────────
function refreshCurrent() {
  const panel = panels[currentTab];
  if (!panel || !connected) return;
  clearTimeout(pollTimers[currentTab]);
  panel.render().catch(err => {
    document.getElementById('tab-content').innerHTML =
      `<div class="error-card">Error loading ${currentTab}: ${err.message}</div>`;
  });
  pollTimers[currentTab] = setTimeout(refreshCurrent, panel.interval);
}

// ── Watchlist ────────────────────────────────────────────────
function setupWatchlist() {
  const STORAGE = 'trading_watchlist';
  let items = JSON.parse(localStorage.getItem(STORAGE) || '[]');

  function save() { localStorage.setItem(STORAGE, JSON.stringify(items)); }

  async function refresh() {
    const container = document.getElementById('watch-items');
    if (items.length === 0) {
      container.innerHTML = '<div class="empty muted">No symbols added</div>';
      return;
    }
    try {
      const resp = await API.getSimplePrice(items.join(','), 'usd');
      const prices = API.unwrapData(resp);
      container.innerHTML = items.map(sym => {
        const data = prices?.[sym] || {};
        const price = data?.usd;
        const change = data?.usd_24h_change;
        const cls = API.pctClass(change);
        return `<div class="watch-item" data-sym="${sym}">
          <span class="sym accent">${sym.toUpperCase()}</span>
          <span>${price ? '$' + API.fmt(price) : '<span class="muted">—</span>'}</span>
          <span class="change ${cls}">${API.fmtPct(change)}</span>
        </div>`;
      }).join('');
    } catch {
      container.innerHTML = '<div class="empty muted">Unable to load prices</div>';
    }
  }

  document.getElementById('watch-add-btn').addEventListener('click', () => {
    const input = document.getElementById('watch-input');
    const sym = input.value.trim().toLowerCase();
    if (sym && !items.includes(sym)) {
      items.push(sym);
      save();
      refresh();
    }
    input.value = '';
    input.focus();
  });

  document.getElementById('watch-input').addEventListener('keydown', e => {
    if (e.key === 'Enter') document.getElementById('watch-add-btn').click();
  });

  setInterval(refresh, 30_000);
  refresh();
}

// ── Ticker Bar ───────────────────────────────────────────────
async function refreshTicker() {
  try {
    const data = await API.getKrakenTicker();
    const tickers = API.unwrapData(data) || [];
    const items = tickers.map(t => {
      const cls = API.pctClass(t.change_pct_24h);
      return `<span class="ticker-item">
        <span class="sym">${t.pair}</span> ${API.fmt(t.price)}
        <span class="${cls}">${API.fmtPct(t.change_pct_24h)}</span>
      </span>`;
    });
    // Duplicate for seamless looping
    document.getElementById('ticker-content').innerHTML = items.join('') + items.join('');
    setTimeout(refreshTicker, 60_000);
  } catch { setTimeout(refreshTicker, 30_000); }
}
refreshTicker();

// ── Boot ─────────────────────────────────────────────────────
init();
