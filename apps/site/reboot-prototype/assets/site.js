document.addEventListener('DOMContentLoaded', () => {
  for (const btn of document.querySelectorAll('.copy-code')) {
    btn.addEventListener('click', async () => {
      const code = btn.closest('.code-window')?.querySelector('pre code')?.textContent;
      if (!code) return;
      try {
        await navigator.clipboard.writeText(code);
        const previous = btn.textContent;
        btn.textContent = document.documentElement.lang === 'ja' ? 'コピー済み ✓' : 'Copied ✓';
        btn.setAttribute('aria-label', btn.textContent);
        setTimeout(() => { btn.textContent = previous; btn.setAttribute('aria-label', previous); }, 1800);
      } catch { btn.textContent = document.documentElement.lang === 'ja' ? 'コピーに失敗' : 'Copy failed'; }
    });
  }
  const input = document.querySelector('[data-doc-search]');
  if (input) {
    const items = [...document.querySelectorAll('[data-search-item]')];
    const empty = document.querySelector('[data-empty-results]');
    const status = document.querySelector('[data-search-status]');
    const update = () => {
      const q = input.value.trim().normalize('NFKC').toLocaleLowerCase();
      let count = 0;
      for (const item of items) {
        const value = (item.dataset.keywords || item.textContent).normalize('NFKC').toLocaleLowerCase();
        const show = !q || value.includes(q);
        item.hidden = !show;
        if (show) count++;
      }
      for (const group of document.querySelectorAll('[data-ref-group]')) {
  const rows = [...group.querySelectorAll('[data-search-item]')];
  group.hidden = !!q && !rows.some(row => !row.hidden);
 }
 if (empty) empty.hidden = count !== 0;
      if (status) status.textContent = q ? (document.documentElement.lang === 'ja' ? `${count}件` : `${count} found`) : '';
    };
    input.addEventListener('input', update);
  }
  for (const link of document.querySelectorAll('.mobile-menu a, .mobile-docs-menu a')) {
    link.addEventListener('click', () => link.closest('details')?.removeAttribute('open'));
  }
});
