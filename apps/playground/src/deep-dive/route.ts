export function deepDiveArticleIdFromUrl(
  href: string,
  articleIds: readonly string[]
): string {
  const requested = new URL(href).searchParams.get("article")
  return requested !== null && articleIds.includes(requested)
    ? requested
    : (articleIds[0] ?? "")
}

export function deepDiveArticleUrl(href: string, articleId: string): string {
  const url = new URL(href)
  url.searchParams.set("article", articleId)
  return url.href
}

export function deepDiveRelativeUrl(articleId: string): string {
  return `../deep-dive/?article=${encodeURIComponent(articleId)}`
}
