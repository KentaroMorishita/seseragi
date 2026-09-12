const maximumLinkedSourceBytes = 64 * 1024

export function playgroundUrlForSource(origin: string, source: string): string {
  const url = new URL(origin)
  url.searchParams.set("source", source)
  return url.toString()
}

export function sourceFromPlaygroundUrl(href: string): string | undefined {
  const source = new URL(href).searchParams.get("source")
  if (source === null) return undefined
  if (new TextEncoder().encode(source).byteLength > maximumLinkedSourceBytes) {
    return undefined
  }
  return source
}
