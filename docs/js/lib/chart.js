// ── Thin Chart.js wrapper ───────────────────────────────────
// Renders sparklines in .sparkline-wrap elements that have `data-spark`.

export function renderSparklines() {
  if (typeof Chart === 'undefined') return;
  document.querySelectorAll('.sparkline-wrap[data-spark]').forEach(el => {
    if (el.dataset.rendered) return;
    el.dataset.rendered = '1';
    const raw = el.dataset.spark;
    if (!raw) return;
    const prices = raw.split(',').map(Number).filter(n => !isNaN(n));
    if (prices.length < 2) return;

    const canvas = document.createElement('canvas');
    el.appendChild(canvas);

    const isUp = prices[prices.length - 1] >= prices[0];
    const color = isUp ? '#00e676' : '#ff3d4f';

    new Chart(canvas, {
      type: 'line',
      data: {
        labels: prices.map((_, i) => i),
        datasets: [{
          data: prices,
          borderColor: color,
          borderWidth: 1,
          pointRadius: 0,
          tension: 0.3,
          fill: false,
        }],
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        animation: false,
        plugins: { legend: { display: false }, tooltip: { enabled: false } },
        scales: {
          x: { display: false },
          y: { display: false },
        },
      },
    });
  });
}

// Call after any panel renders a table with sparkline data.
// Attached as a mutation observer so panels don't need to coordinate.
const observer = new MutationObserver(() => renderSparklines());
observer.observe(document.body, { childList: true, subtree: true });
renderSparklines();
