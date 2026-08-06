import * as API from '../api.js';

let allNews = null;
let currentSource = 'all';

export async function renderNews() {
  const container = document.getElementById('tab-content');
  container.innerHTML = '<div class="loading">Loading news...</div>';

  try {
    const raw = await API.getNews();
    allNews = raw;
    container.innerHTML = renderSourceTabs() + '<div id="news-feed"></div>';
    filterNews();
  } catch (err) {
    container.innerHTML = `<div class="error-card">News unavailable: ${err.message}</div>`;
  }
}

function renderSourceTabs() {
  const sources = ['all', 'reddit', 'alpaca'];
  return `<div class="news-source-tabs">
    ${sources.map(s => `<button class="${s === currentSource ? 'active' : ''}" data-source="${s}">${s.charAt(0).toUpperCase() + s.slice(1)}</button>`).join('')}
  </div>`;

  // Event listeners added outside this string (inline below)
}

function filterNews() {
  const container = document.getElementById('news-feed');
  if (!allNews) return;

  // Source tab click handlers
  document.querySelectorAll('.news-source-tabs button').forEach(btn => {
    btn.onclick = () => {
      currentSource = btn.dataset.source;
      document.querySelectorAll('.news-source-tabs button').forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
      filterNews();
    };
  });

  const items = [];

  if (currentSource === 'all' || currentSource === 'reddit') {
    const reddit = allNews.reddit || {};
    ['wallstreetbets', 'stocks', 'investing'].forEach(sub => {
      const subData = reddit[sub];
      // Reddit source returns either an array of posts, or an {error} object if unconfigured
      const posts = Array.isArray(subData) ? subData : [];
      posts.forEach(post => {
        items.push({
          source: `r/${sub}`,
          headline: post.title || '—',
          url: post.permalink ? `https://reddit.com${post.permalink}` : '#',
          time: post.created_utc ? new Date(post.created_utc * 1000).toLocaleDateString() : '',
          meta: `${post.score || 0} pts · ${post.num_comments || 0} comments`,
        });
      });
    });
  }

  if (currentSource === 'all' || currentSource === 'alpaca') {
    const alpaca = allNews.alpaca;
    // Alpaca returns {news: [...]} when configured, or null when not
    const anews = Array.isArray(alpaca?.news) ? alpaca.news : [];
    anews.forEach(item => {
      items.push({
        source: 'Alpaca',
        headline: item.headline || item.title || '—',
        url: item.url || '#',
        time: item.updated_at || item.created_at || '',
        meta: item.symbols?.join(', ') || item.source || '',
      });
    });
  }

  if (!items.length) {
    container.innerHTML = '<div class="empty muted">No news articles for this source</div>';
    return;
  }

  // Sort by recency heuristic: try to parse time strings
  const displayed = items.slice(0, 100);
  container.innerHTML = displayed.map(item => `
    <div class="news-item">
      <a class="headline" href="${item.url}" target="_blank" rel="noopener">${item.headline}</a>
      <div class="meta">
        <span class="badge badge-info">${item.source}</span>
        ${item.time ? `<span> · ${item.time}</span>` : ''}
        ${item.meta ? `<span> · ${item.meta}</span>` : ''}
      </div>
    </div>
  `).join('');
}
