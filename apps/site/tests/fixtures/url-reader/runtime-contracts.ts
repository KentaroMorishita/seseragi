import assert from "node:assert/strict"
import { writeFileSync } from "node:fs"

type Url = object
type Query = object
type BuildError = { tag: string; value?: unknown }
type Either<T> = { tag: "Left"; value: BuildError } | { tag: "Right"; value: T }
type Maybe<T> = { tag: "Nothing" } | { tag: "Just"; value: T }
interface NavigationValues {
  emptyQuery: Query
  parseUrl(s: string): Either<Url>
  resolveUrl(s: string, u: Url): Either<Url>
  renderUrl(u: Url): string
  parseQuery(s: string): Either<Query>
  renderQuery(q: Query): string
  appendQuery(k: string, v: string, q: Query): Query
  setQuery(k: string, v: string, q: Query): Query
  removeQuery(k: string, q: Query): Query
  queryValues(k: string, q: Query): readonly string[]
  queryEntries(q: Query): readonly (readonly [string, string])[]
  urlQuery(u: Url): Query
  withQuery(q: Query, u: Url): Url
  urlFragment(u: Url): Maybe<string>
  withFragment(s: string, u: Url): Url
  withoutFragment(u: Url): Url
  pathSegments(u: Url): readonly string[]
}
const n = (await import(
  new URL("../../../../../runtime/ts/src/navigation.ts", import.meta.url).href
)) as NavigationValues
const records: Record<string, unknown>[] = []
function check(name: string, actual: unknown, expected: unknown) {
  assert.deepEqual(actual, expected, name)
  records.push({ name, actual, expected })
}
function right<T>(result: Either<T>): T {
  if (result.tag !== "Right")
    throw new Error(`Expected Right: ${JSON.stringify(result)}`)
  return result.value
}
function uriError(name: string, fn: () => unknown) {
  let caught: unknown
  try {
    fn()
  } catch (error) {
    caught = error
  }
  assert(caught instanceof URIError, name)
  records.push({ name, errorName: caught.name, errorMessage: caught.message })
}
const query = right(n.parseQuery("a=1&tag=old&b=2&tag=other&empty="))
check("ordered duplicate pairs", n.queryEntries(query), [
  ["a", "1"],
  ["tag", "old"],
  ["b", "2"],
  ["tag", "other"],
  ["empty", ""],
])
check(
  "set keeps first position",
  n.renderQuery(n.setQuery("tag", "new", query)),
  "a=1&tag=new&b=2&empty="
)
check(
  "set absent appends",
  n.renderQuery(n.setQuery("fresh", "last", query)),
  "a=1&tag=old&b=2&tag=other&empty=&fresh=last"
)
check(
  "append retains duplicates",
  n.renderQuery(n.appendQuery("tag", "end", query)),
  "a=1&tag=old&b=2&tag=other&empty=&tag=end"
)
check(
  "remove deletes all occurrences",
  n.renderQuery(n.removeQuery("tag", query)),
  "a=1&b=2&empty="
)
check(
  "remove absent contents",
  n.renderQuery(n.removeQuery("missing", query)),
  n.renderQuery(query)
)
check("names are case sensitive", n.queryValues("Tag", query), [])
check("missing values", n.queryValues("missing", query), [])
check("present empty value", n.queryValues("empty", query), [""])
check("value order", n.queryValues("tag", query), ["old", "other"])
check(
  "original not mutated",
  n.renderQuery(query),
  "a=1&tag=old&b=2&tag=other&empty="
)
check("query frozen", Object.isFrozen(query), true)
check("entry array frozen", Object.isFrozen(n.queryEntries(query)), true)
check("entry pair frozen", Object.isFrozen(n.queryEntries(query)[0]), true)
check("value array frozen", Object.isFrozen(n.queryValues("tag", query)), true)
check("empty query", n.renderQuery(n.emptyQuery), "")
check("parse empty", n.queryEntries(right(n.parseQuery(""))), [])
check("parse question only", n.queryEntries(right(n.parseQuery("?"))), [])
check(
  "plus and literal plus",
  n.queryEntries(right(n.parseQuery("q=a+b&literal=%2B"))),
  [
    ["q", "a b"],
    ["literal", "+"],
  ]
)
check(
  "empty names and chunks",
  n.queryEntries(right(n.parseQuery("&empty&&=x&"))),
  [
    ["empty", ""],
    ["", "x"],
  ]
)
check(
  "decoded append encoded once",
  n.renderQuery(n.appendQuery("q", "a+b &/", n.emptyQuery)),
  "q=a%2Bb+%26%2F"
)
check(
  "preencoded value encoded again",
  n.renderQuery(n.appendQuery("q", "%20", n.emptyQuery)),
  "q=%2520"
)
check(
  "double question current behavior",
  n.queryEntries(right(n.parseQuery("??q=x"))),
  [["q", "x"]]
)
check(
  "native double question differs",
  [...new URLSearchParams("??q=x")],
  [["?q", "x"]]
)
check("query invalid triplet", n.parseQuery("q=%ZZ"), {
  tag: "Left",
  value: { tag: "InvalidPercentEncoding", value: { offset: 2 } },
})
check("offset UTF16 after stripped marker", n.parseQuery("?q=😀%ZZ"), {
  tag: "Left",
  value: { tag: "InvalidPercentEncoding", value: { offset: 4 } },
})
check(
  "query byte invalid decoded replacement",
  n.queryValues("q", right(n.parseQuery("q=%FF"))),
  ["�"]
)
check(
  "replacement reencoded",
  n.renderQuery(right(n.parseQuery("q=%FF"))),
  "q=%EF%BF%BD"
)
check("relative URL rejected", n.parseUrl("/relative"), {
  tag: "Left",
  value: { tag: "InvalidUrl", value: { offset: 0 } },
})
check("nonhttp scheme rejected", n.parseUrl("ftp://example.test/"), {
  tag: "Left",
  value: { tag: "UnsupportedUrlScheme", value: "ftp" },
})
check("userinfo rejected", n.parseUrl("https://user:pass@example.test/"), {
  tag: "Left",
  value: { tag: "UrlContainsUserInfo" },
})
check(
  "percent preflight before scheme and userinfo",
  n.parseUrl("ftp://user@example.test/%ZZ"),
  {
    tag: "Left",
    value: { tag: "InvalidPercentEncoding", value: { offset: 24 } },
  }
)
const base = right(
  n.parseUrl("HTTPS://EXAMPLE.TEST:443/a/../search?q=a%20b~#old")
)
check(
  "normalization retains query spelling",
  n.renderUrl(base),
  "https://example.test/search?q=a%20b~#old"
)
check(
  "query roundtrip normalizes encoding",
  n.renderUrl(n.withQuery(n.urlQuery(base), base)),
  "https://example.test/search?q=a+b%7E#old"
)
const detached = n.setQuery("q", "new", n.urlQuery(base))
check(
  "detached query does not modify URL",
  n.renderUrl(base),
  "https://example.test/search?q=a%20b~#old"
)
check(
  "withQuery replaces query",
  n.renderUrl(n.withQuery(detached, base)),
  "https://example.test/search?q=new#old"
)
check(
  "empty query removes marker",
  n.renderUrl(n.withQuery(n.emptyQuery, base)),
  "https://example.test/search#old"
)
check("url frozen", Object.isFrozen(base), true)
check(
  "fragment decoded",
  n.urlFragment(right(n.parseUrl("https://example.test/#section%201"))),
  { tag: "Just", value: "section 1" }
)
check(
  "fragment plus remains plus",
  n.urlFragment(right(n.parseUrl("https://example.test/#a+b"))),
  { tag: "Just", value: "a+b" }
)
check(
  "bare fragment absent",
  n.urlFragment(right(n.parseUrl("https://example.test/#"))),
  { tag: "Nothing" }
)
check(
  "withFragment encodes text",
  n.renderUrl(n.withFragment("results 2+#", base)),
  "https://example.test/search?q=a%20b~#results%202%2B%23"
)
check(
  "withFragment empty removes",
  n.renderUrl(n.withFragment("", base)),
  "https://example.test/search?q=a%20b~"
)
check(
  "withoutFragment preserves query",
  n.renderUrl(n.withoutFragment(base)),
  "https://example.test/search?q=a%20b~"
)
check(
  "withoutFragment repeat same visible URL",
  n.renderUrl(n.withoutFragment(n.withoutFragment(base))),
  n.renderUrl(n.withoutFragment(base))
)
const fileBase = right(n.parseUrl("https://example.test/catalog/item"))
const directoryBase = right(n.parseUrl("https://example.test/catalog/item/"))
check(
  "file base",
  n.renderUrl(right(n.resolveUrl("next", fileBase))),
  "https://example.test/catalog/next"
)
check(
  "directory base",
  n.renderUrl(right(n.resolveUrl("next", directoryBase))),
  "https://example.test/catalog/item/next"
)
check(
  "absolute resolve may change origin",
  n.renderUrl(right(n.resolveUrl("https://outside.test/search", base))),
  "https://outside.test/search"
)
check(
  "network reference may change origin",
  n.renderUrl(right(n.resolveUrl("//outside.test/search", base))),
  "https://outside.test/search"
)
const malformedFragment = right(n.parseUrl("https://example.test/#%FF"))
check(
  "triplet-valid fragment URL parses",
  n.renderUrl(malformedFragment),
  "https://example.test/#%FF"
)
uriError("urlFragment malformed UTF8 throws", () =>
  n.urlFragment(malformedFragment)
)
uriError("pathSegments malformed UTF8 throws", () =>
  n.pathSegments(right(n.parseUrl("https://example.test/%FF")))
)
writeFileSync(
  process.argv[2],
  JSON.stringify(
    { status: "passed", assertions: records.length, records },
    null,
    2
  )
)
console.log(JSON.stringify({ status: "passed", assertions: records.length }))
