import * as API from '../api.js';

export async function renderCrypto() {
  const container = document.getElementById('tab-content');
  container.innerHTML = '<div class="loading">Loading crypto data...</div>';

  try {
    const [coins, krakenTrending] = await Promise.all([
      API.getTopCoins(50),
      API.getKrakenTrending(10),
    ]);

    const data = API.unwrapData(coins) || [];
    const krakenData = API.unwrapData(krakenTrending) || [];

    let html = renderCoinTable(data);
    html += '<div class="section-title" style="margin-top:16px">Kraken Trending</div>';
    html += renderKrakenStrip(krakenData);

    container.innerHTML = html;

    // Sorting
    setupTableSorting(container.querySelector('#crypto-table'));

  } catch (err) {
    container.innerHTML = `<div class="error-card">Crypto data unavailable: ${err.message}</div>`;
  }
}

function renderCoinTable(coins) {
  if (!coins.length) return '<div class="empty muted">No coin data</div>';

  const rows = coins.map((c, i) => {
    const p24 = c.price_change_percentage_24h;
    const p7d = c.price_change_percentage_7d_in_currency;
    const p30d = c.price_change_percentage_30d_in_currency;
    const cls24 = API.pctClass(p24);
    const cls7d = API.pctClass(p7d);
    const cls30d = API.pctClass(p30d);
    const spark = c.sparkline_in_7d?.price || [];

    return `<tr class="${cls24}" data-sort-price="${c.current_price || 0}" data-sort-pct="${p24 || 0}" data-sort-mcap="${c.market_cap || 0}">
      <td class="muted">${i + 1}</td>
      <td><span class="accent">${(c.symbol || '').toUpperCase()}</span></td>
      <td class="num">$${API.fmt(c.current_price)}</td>
      <td class="num ${cls24}">${API.fmtPct(p24)}</td>
      <td class="num ${cls7d}">${API.fmtPct(p7d)}</td>
      <td class="num ${cls30d}">${API.fmtPct(p30d)}</td>
      <td class="num muted">$${API.fmt(c.market_cap)}</td>
      <td class="sparkline-wrap" data-spark="${spark.join(',')}"></td>
    </tr>`;
  }).join('');

  return `<table id="crypto-table" class="table-terminal">
    <thead><tr>
      <th>#</th>
      <th>Symbol</th>
      <th class="num" data-sort="price">Price</th>
      <th class="num" data-sort="pct">24h</th>
      <th class="num">7d</th>
      <th class="num">30d</th>
      <th class="num" data-sort="mcap">Market Cap</th>
      <th>7d Chart</th>
    </tr></thead>
    <tbody>${rows}</tbody>
  </table>`;
}

function renderKrakenStrip(items) {
  if (!items.length) return '<div class="empty muted">No Kraken data</div>';
  return `<div style="display:flex;gap:12px;flex-wrap:wrap">
    ${items.slice(0, 8).map(t => {
      const cls = API.pctClass(t.price_change_percentage_24h);
      return `<span class="badge badge-info">${t.symbol || t.id} <span class="${cls}">${API.fmtPct(t.price_change_percentage_24h)}</span></span>`;
    }).join('')}
  </div>`;
}

function setupTableSorting(table) {
  if (!table) return;
  table.querySelectorAll('th[data-sort]').forEach(th => {
    th.addEventListener('click', () => {
      const key = th.dataset.sort;
      const tbody = table.querySelector('tbody');
      const rows = [...tbody.querySelectorAll('tr')];
      const isAsc = th.classList.toggle('asc');
      table.querySelectorAll('th').forEach(h => h.classList.remove('sorted'));
      th.classList.add('sorted');
      rows.sort((a, b) => {
        const av = parseFloat(a.dataset[`sort${key}`] || 0);
        const bv = parseFloat(b.dataset[`sort${key}`] || 0);
        return isAsc ? av - bv : bv - av;
      });
      rows.forEach(r => tbody.appendChild(r));
    });
  });
}
