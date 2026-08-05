import * as API from '../api.js';

export async function renderOptions() {
  const container = document.getElementById('tab-content');
  container.innerHTML = `
    <div class="filter-form">
      <div><label>Symbols</label><input type="text" id="opt-symbols" value="AAPL,MSFT,GOOGL,NVDA,TSLA,META" style="width:220px"></div>
      <div><label>Side</label><select id="opt-side"><option>call</option><option>put</option><option>both</option></select></div>
      <div><label>Min DTE</label><input type="number" id="opt-mindte" value="7" style="width:55px"></div>
      <div><label>Max DTE</label><input type="number" id="opt-maxdte" value="60" style="width:55px"></div>
      <div><label>Limit</label><input type="number" id="opt-limit" value="20" style="width:55px"></div>
      <div><label>&nbsp;</label><button id="opt-search">Search</button></div>
    </div>
    <div id="options-table-wrap"><div class="loading">Enter parameters and click Search</div></div>
  `;

  document.getElementById('opt-search').addEventListener('click', doSearch);
  doSearch(); // initial load
}

async function doSearch() {
  const wrap = document.getElementById('options-table-wrap');
  wrap.innerHTML = '<div class="loading">Scanning options...</div>';

  try {
    const params = {
      symbols: document.getElementById('opt-symbols').value,
      side: document.getElementById('opt-side').value,
      min_dte: parseInt(document.getElementById('opt-mindte').value) || 7,
      max_dte: parseInt(document.getElementById('opt-maxdte').value) || 60,
      limit: parseInt(document.getElementById('opt-limit').value) || 20,
      range: '1mo',
      interval: '1d',
    };

    const resp = await API.getOptionsRecommendations(params);
    const results = API.unwrapData(resp) || [];

    if (!results.length) {
      wrap.innerHTML = '<div class="empty muted">No options contracts found for those parameters</div>';
      return;
    }

    wrap.innerHTML = `<table class="table-terminal">
      <thead><tr>
        <th>Symbol</th><th>Contract</th><th>Side</th><th class="num">Strike</th>
        <th class="num">Premium</th><th class="num">DTE</th><th class="num">Delta</th>
        <th class="num">IV</th><th class="num">Score</th>
      </tr></thead>
      <tbody>${results.map(r => {
        const cls = (r.side === 'call' || r.side === 'Call') ? 'up' : 'down';
        const scorePct = Math.min(Math.max((r.score || 0) * 100, 0), 100);
        return `<tr>
          <td class="accent">${r.symbol || '—'}</td>
          <td class="muted">${r.contract || '—'}</td>
          <td class="${cls}">${r.side || '—'}</td>
          <td class="num">$${API.fmt(r.strike, 1)}</td>
          <td class="num">$${API.fmt(r.premium, 2)}</td>
          <td class="num">${Math.round(r.dte_days || 0)}d</td>
          <td class="num">${r.delta != null ? r.delta.toFixed(3) : '—'}</td>
          <td class="num">${API.fmtPct(r.implied_vol)}</td>
          <td class="num">
            <span class="score-bar"><span class="score-bar-fill" style="width:${scorePct}%"></span></span>
            ${(r.score || 0).toFixed(3)}
          </td>
        </tr>`;
      }).join('')}</tbody>
    </table>`;

  } catch (err) {
    wrap.innerHTML = `<div class="error-card">Options scan failed: ${err.message}</div>`;
  }
}
