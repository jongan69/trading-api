import * as API from '../api.js';

export async function renderStocks() {
  const container = document.getElementById('tab-content');
  container.innerHTML = '<div class="loading">Loading stock data...</div>';

  try {
    const trending = await API.getTrendingStocks(50);
    const symbols = API.unwrapData(trending) || [];

    let html = '<div class="section-title">Trending Tickers</div>';
    html += renderTrendingGrid(symbols);

    // Try Yahoo rankings if symbols are available
    if (symbols.length) {
      try {
        const top = symbols.slice(0, 10).join(',');
        const rankResp = await API.getYahooRank(top, '1mo', '1d');
        const results = API.unwrapData(rankResp) || [];
        if (results.length) {
          html += '<div class="section-title" style="margin-top:16px">Yahoo Rankings (Top 10)</div>';
          html += renderRankTable(results);
        }
      } catch { /* Yahoo may be unavailable, skip */ }
    }

    container.innerHTML = html;
  } catch (err) {
    container.innerHTML = `<div class="error-card">Stock data unavailable: ${err.message}</div>`;
  }
}

function renderTrendingGrid(symbols) {
  if (!symbols.length) return '<div class="empty muted">No trending stocks</div>';
  return `<div style="display:flex;gap:6px;flex-wrap:wrap;padding:8px 0">
    ${symbols.slice(0, 50).map(s => `<span class="badge badge-info">${s}</span>`).join('')}
  </div>`;
}

function renderRankTable(results) {
  return `<table class="table-terminal">
    <thead><tr><th>Symbol</th><th class="num">Sharpe</th><th class="num">Sortino</th><th class="num">Calmar</th><th class="num">Vol</th><th class="num">Score</th></tr></thead>
    <tbody>${results.map(r => {
      const m = r.metrics || {};
      const score = m.composite_score || 0;
      const scorePct = Math.min(Math.max(score * 100, 0), 100);
      return `<tr>
        <td class="accent">${r.symbol || '—'}</td>
        <td class="num">${(m.sharpe || 0).toFixed(2)}</td>
        <td class="num">${(m.sortino || 0).toFixed(2)}</td>
        <td class="num">${(m.calmar || 0).toFixed(2)}</td>
        <td class="num">${API.fmtPct(m.volatility)}</td>
        <td class="num">
          <span class="score-bar"><span class="score-bar-fill" style="width:${scorePct}%"></span></span>
          ${score.toFixed(3)}
        </td>
      </tr>`;
    }).join('')}</tbody>
  </table>`;
}
