import * as API from '../api.js';

export async function renderStocks() {
  const container = document.getElementById('tab-content');
  container.innerHTML = '<div class="loading">Loading stock data...</div>';

  try {
    const [trending, groups] = await Promise.all([
      API.getTrendingStocks(25),
      API.getGroup(25),
    ]);

    const symbols = API.unwrapData(trending) || [];
    const groupData = API.unwrapData(groups) || [];

    let html = '<div class="section-title">Trending Tickers</div>';
    html += renderTrendingGrid(symbols);
    html += '<div class="section-title" style="margin-top:16px">Industry Groups</div>';
    html += renderGroupTable(groupData);

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

function renderGroupTable(groups) {
  if (!groups.length) return '<div class="empty muted">No group data</div>';

  // Finviz returns header→string maps; extract keys from first row
  const headers = Object.keys(groups[0] || {});
  const interesting = headers.filter(h =>
    /name|market cap|p\/e|change|volume|performance/i.test(h)
  );

  return `<table class="table-terminal">
    <thead><tr>${interesting.map(h => `<th>${h}</th>`).join('')}</tr></thead>
    <tbody>${groups.map(row => `<tr>${interesting.map(h => {
      const val = row[h] || '—';
      const cls = (typeof val === 'string' && val.includes('%') && !val.startsWith('-'))
        ? 'up' : (typeof val === 'string' && val.startsWith('-') ? 'down' : '');
      return `<td class="num ${cls}">${val}</td>`;
    }).join('')}</tr>`).join('')}</tbody>
  </table>`;
}
