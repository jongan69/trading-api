import * as API from '../api.js';

export async function renderForex() {
  const container = document.getElementById('tab-content');
  container.innerHTML = '<div class="loading">Loading forex data...</div>';

  try {
    const resp = await API.getForex(30);
    const data = API.unwrapData(resp) || [];

    if (!data.length) {
      container.innerHTML = '<div class="empty muted">No forex data available</div>';
      return;
    }

    // Finviz forex returns dynamic header→string rows
    const headers = Object.keys(data[0] || {});
    const interesting = headers.filter(h => /pair|price|change|bid|ask|high|low/i.test(h));
    const displayHeaders = interesting.length ? interesting : headers.slice(0, 6);

    container.innerHTML = `<table class="table-terminal">
      <thead><tr>${displayHeaders.map(h => `<th>${h}</th>`).join('')}</tr></thead>
      <tbody>${data.map(row => `<tr>${displayHeaders.map(h => {
        const val = row[h] || '—';
        const cls = (typeof val === 'string' && val.includes('%') && !val.startsWith('-'))
          ? 'up' : (typeof val === 'string' && val.startsWith('-') ? 'down' : '');
        return `<td class="num ${cls}">${val}</td>`;
      }).join('')}</tr>`).join('')}</tbody>
    </table>`;
  } catch (err) {
    container.innerHTML = `<div class="error-card">Forex data unavailable: ${err.message}</div>`;
  }
}
