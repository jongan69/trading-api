import * as API from '../api.js';

export async function renderMarketOverview() {
  const tiles = document.getElementById('market-tiles');
  const gainers = document.getElementById('market-gainers');

  try {
    const [overview, topGainers] = await Promise.all([
      API.getMarketOverview(),
      API.getTopGainers(5),
    ]);

    const data = API.unwrapData(overview);
    tiles.innerHTML = data ? `
      <div class="stat-tile">
        <div class="label">Total Market Cap</div>
        <div class="value">$${API.fmt(data.total_market_cap)}</div>
      </div>
      <div class="stat-tile">
        <div class="label">24h Volume</div>
        <div class="value">$${API.fmt(data.total_volume)}</div>
      </div>
      <div class="stat-tile">
        <div class="label">BTC Dominance</div>
        <div class="value">${API.fmtPct(data.bitcoin_dominance)}</div>
      </div>
      <div class="stat-tile">
        <div class="label">Active Coins (Top)</div>
        <div class="value">${Object.keys(data.market_cap_percentage || {}).length || '—'}</div>
      </div>
    ` : '<div class="empty muted">No market data</div>';

    const coinList = API.unwrapData(topGainers) || [];
    gainers.innerHTML = coinList.length > 0
      ? coinList.slice(0, 5).map(c => {
          const change = c.price_change_percentage_24h;
          const cls = API.pctClass(change);
          return `<div class="gainer-item">
            <span class="sym accent">${(c.symbol || '').toUpperCase()}</span>
            <span class="muted">$${API.fmt(c.current_price)}</span>
            <span class="change ${cls}">${API.fmtPct(change)}</span>
          </div>`;
        }).join('')
      : '<div class="empty muted">—</div>';

  } catch (err) {
    tiles.innerHTML = '<div class="error-card">Market data unavailable</div>';
    gainers.innerHTML = '';
  }
}
