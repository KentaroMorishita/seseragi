function escapeText(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
}

function update(expanded: boolean): boolean {
  return !expanded
}

// Pure HTML snapshots of the same visible states; this does not use the DOM.
function view(expanded: boolean): string {
  const title = escapeText("Release <notes>")
  const detail = escapeText("Search & navigation improvements")
  const hidden = expanded ? "" : " hidden"
  const label = expanded ? "Hide details" : "Show details"
  return `<section class="release-card"><h2>${title}</h2><p${hidden}>${detail}</p><button id="toggle-details" aria-expanded="${expanded}" type="button">${label}</button></section>`
}

console.log(view(false))
console.log(view(update(false)))
console.log(view(update(true)))
export {}
