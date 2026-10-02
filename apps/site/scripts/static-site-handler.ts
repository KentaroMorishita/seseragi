import { realpathSync, statSync } from "node:fs"
import { extname, resolve, sep } from "node:path"

type SiteConfiguration = {
  trailingSlash?: boolean
  headers: Array<{
    source: string
    headers: Array<{ key: string; value: string }>
  }>
}

// Implements this static site's catch-all headers and directory-index routes.
// It intentionally does not emulate arbitrary Vercel rewrites or routing rules.
export function staticSiteHandler(
  output: string,
  configuration: SiteConfiguration
) {
  const root = realpathSync(output)
  const headerRules = configuration.headers.filter(
    ({ source }) => source === "/(.*)"
  )
  const headers = Object.fromEntries(
    headerRules.flatMap(({ headers }) =>
      headers.map(({ key, value }) => [key, value])
    )
  )
  const notFound = () => new Response("Not found", { status: 404, headers })

  return (request: Request): Response => {
    const url = new URL(request.url)
    let pathname: string
    try {
      pathname = decodeURIComponent(url.pathname)
    } catch {
      return notFound()
    }
    if (pathname.includes("\\") || pathname.includes("\0")) return notFound()
    const relative = pathname.endsWith("/")
      ? `${pathname.slice(1)}index.html`
      : pathname.slice(1)
    const path = resolve(root, relative || "index.html")
    if (!path.startsWith(`${root}${sep}`)) return notFound()

    if (
      configuration.trailingSlash === true &&
      !pathname.endsWith("/") &&
      extname(pathname) === ""
    ) {
      return new Response(null, {
        status: 308,
        headers: { ...headers, Location: `${url.pathname}/${url.search}` },
      })
    }
    try {
      if (
        !statSync(path).isFile() ||
        !realpathSync(path).startsWith(`${root}${sep}`)
      ) {
        return notFound()
      }
    } catch {
      return notFound()
    }
    return new Response(Bun.file(path), { headers })
  }
}
