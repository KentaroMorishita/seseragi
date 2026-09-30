import assert from "node:assert/strict"

function visibleText(html: string): string {
  return html
    .replace(/<[^>]+>/gu, "")
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&quot;", '"')
    .replaceAll("&#39;", "'")
    .replaceAll("&amp;", "&")
    .replace(/\s+/gu, " ")
    .trim()
}

export function pageTitle(html: string): string {
  const heading = html.match(/<h1\b[^>]*>([\s\S]*?)<\/h1>/u)
  assert.ok(heading, "reference page must have a primary heading")
  return visibleText(heading[1])
}

// Page-name references must use the destination's localized title. Navigation,
// buttons, section anchors and descriptive calls to action are separate UI.
export function assertReferenceLinkTitles(
  html: string,
  titles: ReadonlyMap<string, string>,
  route: string
): number {
  const article = html.match(/<article\b[^>]*>([\s\S]*?)<\/article>/u)
  assert.ok(article, `${route}: missing article`)
  let checked = 0
  const links = [
    ...article[1].matchAll(
      /<li\b[^>]*>\s*<a\b([^>]*)>([\s\S]*?)<\/a>\s*<\/li>/gu
    ),
    ...article[1].matchAll(
      /<p\b[^>]*>\s*<a\b([^>]*)>([\s\S]*?)<\/a>\s*<span>\s*(?:：|—)/gu
    ),
  ]
  for (const link of links) {
    const destination = link[1].match(/\bhref="([^"]+)"/u)?.[1]
    if (!destination) continue
    const title = titles.get(destination)
    if (!title) continue
    assert.equal(
      visibleText(link[2]),
      title,
      `${route}: page-name link to ${destination} must match its heading`
    )
    checked++
  }
  return checked
}
