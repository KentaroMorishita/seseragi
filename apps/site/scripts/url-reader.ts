import { canonicalExample } from "./canonical-example"

export const urlReaderCases = [
  {
    slug: "build-filter-query",
    output: "q=tea+%26+cake\nq=tea+%26+cake&tag=books&tag=music\n",
  },
  {
    slug: "check-absolute-link",
    output: "https://example.test/search\nAbsolute link rejected\n",
  },
  {
    slug: "clear-page-filter",
    output: "q=tea&tag=books\nq=tea&tag=books\n",
  },
  {
    slug: "parse-filter-query",
    output:
      "q=tea+cake&literal=%2B&path=%2F&empty=\nq=tea+cake&literal=%2B&path=%2F&empty=\n",
  },
  {
    slug: "read-selected-tags",
    output:
      "books, music\nMissing count: 0\nEmpty count: 1\ntag=books | q=tea cake | tag=music | empty=\n",
  },
  {
    slug: "remove-section",
    output:
      "https://example.test/search?q=tea\nhttps://example.test/search?q=tea\n",
  },
  {
    slug: "render-normalized-link",
    output:
      "https://example.test/search?q=tea%20cake~#results\nhttps://example.test/search?q=tea+cake%7E#results\n",
  },
  {
    slug: "replace-filter",
    output: "q=tea&tag=sale&page=3\nq=tea&tag=books&page=3&tag=music\n",
  },
  {
    slug: "replace-link-query",
    output:
      "tag=books&page=3\nhttps://example.test/search?q=tea#results\nhttps://example.test/search#results\nhttps://example.test/search?tag=books&page=3#results\n",
  },
  {
    slug: "resolve-search-link",
    output:
      "https://example.test/search?tag=books\nhttps://example.test/catalog/search?tag=books\n",
  },
  {
    slug: "set-read-section",
    output:
      "https://example.test/search?q=tea#results%202\nSection: results 2\nNo section\n",
  },
  {
    slug: "share-search-link",
    output:
      "https://example.test/search?q=tea+%26+cake&tag=books&tag=music&tag=sale#results\n",
  },
  {
    slug: "url-build-errors",
    output:
      "https://example.test/search\nInvalid URL at 0\nUnsupported scheme: ftp\nRemove login details\nInvalid percent escape at 21\n",
  },
] as const

export const urlReaderRoutes = [
  {
    identity: "std/web/navigation",
    module: "std/web/navigation",
    namespace: "module",
    kind: "module",
    name: "std/web/navigation",
    slug: "module",
    example: "share-search-link",
    route: "/docs/library/web/navigation/",
  },
  {
    identity: "std/web/navigation::Url",
    module: "std/web/navigation",
    namespace: "type",
    kind: "opaque-type",
    name: "Url",
    slug: "url",
    example: "share-search-link",
    route: "/docs/library/web/navigation/opaque-type/url/",
  },
  {
    identity: "std/web/navigation::Query",
    module: "std/web/navigation",
    namespace: "type",
    kind: "opaque-type",
    name: "Query",
    slug: "query",
    example: "build-filter-query",
    route: "/docs/library/web/navigation/opaque-type/query/",
  },
  {
    identity: "std/web/navigation::UrlBuildError",
    module: "std/web/navigation",
    namespace: "type",
    kind: "opaque-type",
    name: "UrlBuildError",
    slug: "urlbuilderror",
    example: "url-build-errors",
    route: "/docs/library/web/navigation/opaque-type/urlbuilderror/",
  },
  {
    identity: "std/web/navigation::parseUrl",
    module: "std/web/navigation",
    namespace: "value",
    kind: "function",
    name: "parseUrl",
    slug: "parseurl",
    example: "check-absolute-link",
    route: "/docs/library/web/navigation/function/parseurl/",
  },
  {
    identity: "std/web/navigation::resolveUrl",
    module: "std/web/navigation",
    namespace: "value",
    kind: "function",
    name: "resolveUrl",
    slug: "resolveurl",
    example: "resolve-search-link",
    route: "/docs/library/web/navigation/function/resolveurl/",
  },
  {
    identity: "std/web/navigation::renderUrl",
    module: "std/web/navigation",
    namespace: "value",
    kind: "function",
    name: "renderUrl",
    slug: "renderurl",
    example: "render-normalized-link",
    route: "/docs/library/web/navigation/function/renderurl/",
  },
  {
    identity: "std/web/navigation::emptyQuery",
    module: "std/web/navigation",
    namespace: "value",
    kind: "value",
    name: "emptyQuery",
    slug: "emptyquery",
    example: "build-filter-query",
    route: "/docs/library/web/navigation/value/emptyquery/",
  },
  {
    identity: "std/web/navigation::parseQuery",
    module: "std/web/navigation",
    namespace: "value",
    kind: "function",
    name: "parseQuery",
    slug: "parsequery",
    example: "parse-filter-query",
    route: "/docs/library/web/navigation/function/parsequery/",
  },
  {
    identity: "std/web/navigation::appendQuery",
    module: "std/web/navigation",
    namespace: "value",
    kind: "function",
    name: "appendQuery",
    slug: "appendquery",
    example: "build-filter-query",
    route: "/docs/library/web/navigation/function/appendquery/",
  },
  {
    identity: "std/web/navigation::setQuery",
    module: "std/web/navigation",
    namespace: "value",
    kind: "function",
    name: "setQuery",
    slug: "setquery",
    example: "replace-filter",
    route: "/docs/library/web/navigation/function/setquery/",
  },
  {
    identity: "std/web/navigation::removeQuery",
    module: "std/web/navigation",
    namespace: "value",
    kind: "function",
    name: "removeQuery",
    slug: "removequery",
    example: "clear-page-filter",
    route: "/docs/library/web/navigation/function/removequery/",
  },
  {
    identity: "std/web/navigation::queryValues",
    module: "std/web/navigation",
    namespace: "value",
    kind: "function",
    name: "queryValues",
    slug: "queryvalues",
    example: "read-selected-tags",
    route: "/docs/library/web/navigation/function/queryvalues/",
  },
  {
    identity: "std/web/navigation::queryEntries",
    module: "std/web/navigation",
    namespace: "value",
    kind: "function",
    name: "queryEntries",
    slug: "queryentries",
    example: "read-selected-tags",
    route: "/docs/library/web/navigation/function/queryentries/",
  },
  {
    identity: "std/web/navigation::renderQuery",
    module: "std/web/navigation",
    namespace: "value",
    kind: "function",
    name: "renderQuery",
    slug: "renderquery",
    example: "parse-filter-query",
    route: "/docs/library/web/navigation/function/renderquery/",
  },
  {
    identity: "std/web/navigation::urlQuery",
    module: "std/web/navigation",
    namespace: "value",
    kind: "function",
    name: "urlQuery",
    slug: "urlquery",
    example: "replace-link-query",
    route: "/docs/library/web/navigation/function/urlquery/",
  },
  {
    identity: "std/web/navigation::withQuery",
    module: "std/web/navigation",
    namespace: "value",
    kind: "function",
    name: "withQuery",
    slug: "withquery",
    example: "replace-link-query",
    route: "/docs/library/web/navigation/function/withquery/",
  },
  {
    identity: "std/web/navigation::urlFragment",
    module: "std/web/navigation",
    namespace: "value",
    kind: "function",
    name: "urlFragment",
    slug: "urlfragment",
    example: "set-read-section",
    route: "/docs/library/web/navigation/function/urlfragment/",
  },
  {
    identity: "std/web/navigation::withFragment",
    module: "std/web/navigation",
    namespace: "value",
    kind: "function",
    name: "withFragment",
    slug: "withfragment",
    example: "set-read-section",
    route: "/docs/library/web/navigation/function/withfragment/",
  },
  {
    identity: "std/web/navigation::withoutFragment",
    module: "std/web/navigation",
    namespace: "value",
    kind: "function",
    name: "withoutFragment",
    slug: "withoutfragment",
    example: "remove-section",
    route: "/docs/library/web/navigation/function/withoutfragment/",
  },
] as const

// Pure URL operations still belong to a browser-only standard module. Keep
// these exact standalone Playground seeds in their separate web-target package.
export function urlReaderExamples(playgroundUrl: string) {
  return urlReaderCases.flatMap(({ slug }) => [
    canonicalExample(
      `url-reader-${slug}`,
      `apps/site/examples/projects/url-reader/src/${slug}.ssrg`,
      playgroundUrl
    ),
    canonicalExample(
      `url-reader-${slug}-ts`,
      `apps/site/examples/comparisons/url-reader/${slug}.ts`,
      playgroundUrl,
      false,
      "typescript"
    ),
  ])
}
