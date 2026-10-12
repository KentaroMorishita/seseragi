export type SearchEntry = {
  url: string
  title: string
  context: string
  description: string
  body: string
}

export const resultLimit = 40

const normalize = (value: string) => value.normalize("NFKC").toLowerCase()

export function prepareSearch(entries: SearchEntry[]) {
  const prepared = entries.map((entry, order) => ({
    entry,
    order,
    title: normalize(entry.title),
    context: normalize(entry.context),
    text: normalize(
      [entry.title, entry.context, entry.description, entry.body].join(" ")
    ),
  }))
  return (query: string) => {
    const value = normalize(query.slice(0, 256)).trim()
    const tokens = value.split(/\s+/u).filter(Boolean)
    if (!tokens.length) return { entries: [], total: 0 }
    const matches = prepared
      .filter(({ text }) => tokens.every((token) => text.includes(token)))
      .map((item) => ({
        ...item,
        score:
          (item.title === value ? 100 : 0) +
          tokens.reduce(
            (score, token) =>
              score +
              (item.title === token
                ? 40
                : item.title.includes(token)
                  ? 10
                  : 0) +
              (item.context.includes(token) ? 5 : 0),
            0
          ),
      }))
      .sort((a, b) => b.score - a.score || a.order - b.order)
    return {
      entries: matches.slice(0, resultLimit).map(({ entry }) => entry),
      total: matches.length,
    }
  }
}
