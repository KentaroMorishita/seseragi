import { prepareSearch, type SearchEntry } from "./search-engine.js"

const form = document.querySelector<HTMLFormElement>("#docs-search-form")
const query = document.querySelector<HTMLInputElement>("#docs-search-query")
const results = document.querySelector<HTMLOListElement>("#docs-search-results")
const status = document.querySelector<HTMLElement>("#docs-search-count")
const total = document.querySelector<HTMLElement>("#docs-search-total")
const empty = document.querySelector<HTMLElement>("#docs-search-empty")
const more = document.querySelector<HTMLElement>("#docs-search-more")
const unavailable = document.querySelector<HTMLElement>(
  "#docs-search-unavailable"
)

async function start() {
  if (!form || !query || !results || !status || !total || !empty || !more)
    return
  const locale = document.documentElement.lang === "ja" ? "ja" : "en"
  // Module loading uses script-src 'self'. No API, analytics, or fetch is needed;
  // the documentation's connect-src 'none' policy remains in force.
  const { default: entries } = (await import(
    `./search-index-${locale}.js`
  )) as { default: SearchEntry[] }
  const find = prepareSearch(entries)
  query.maxLength = 256
  query.value = (new URL(location.href).searchParams.get("q") ?? "").slice(
    0,
    256
  )
  const update = () => {
    const found = find(query.value)
    const fragment = document.createDocumentFragment()
    for (const entry of found.entries) {
      const item = document.createElement("li")
      const link = document.createElement("a")
      link.href = entry.url
      link.textContent = entry.title
      const context = document.createElement("span")
      context.className = "search-context"
      context.textContent = entry.context
      const description = document.createElement("p")
      description.textContent = entry.description
      item.append(context, link, description)
      fragment.append(item)
    }
    results.replaceChildren(fragment)
    const active = Boolean(query.value.trim())
    total.textContent = String(found.total)
    status.hidden = !active
    empty.hidden = !active || found.total !== 0
    more.hidden = found.total <= found.entries.length
    const url = new URL(location.href)
    if (active) url.searchParams.set("q", query.value)
    else url.searchParams.delete("q")
    history.replaceState(null, "", url)
  }
  query.addEventListener("input", (event) => {
    if (!(event instanceof InputEvent && event.isComposing)) update()
  })
  query.addEventListener("compositionend", update)
  form.addEventListener("submit", (event) => {
    event.preventDefault()
    update()
    results.querySelector<HTMLAnchorElement>("a")?.focus()
  })
  update()
  form.hidden = false
}

void start().catch(() => {
  if (unavailable) unavailable.hidden = false
})
