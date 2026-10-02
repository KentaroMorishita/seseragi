import { canonicalExample } from "./canonical-example"

// Source files, Playground seeds, and expected observations share one mapping.
// Web compilation and in-process provider tests are not real-browser evidence.
export const webReaderCases = [
  {
    slug: "html-module",
    target: "process",
    output: "<p>Release &lt;notes&gt;</p>\n",
  },
  {
    slug: "cards",
    target: "process",
    output:
      '<section class="release-card"><h2>Release &lt;notes&gt;</h2><p>Search &amp; navigation improvements</p></section><section class="release-card"><h2>Next release</h2><p>Two smaller fixes</p></section>\n',
    typescript: "cards",
  },
  {
    slug: "elementprops",
    target: "process",
    output:
      '<p>Release details</p>\n<p class="release-detail" title="Read release notes">Release details</p>\n<p hidden>Release details</p>\n',
  },
  {
    slug: "intochildren",
    target: "process",
    output:
      "<section>Release</section>\n<section></section>\n<section><p>Details</p></section>\n<section><h2>Release</h2><p>Details</p></section>\n<section><h2>Release</h2><p>Details</p></section>\n<section>Release<p>Details</p></section>\n",
  },
  {
    slug: "h2",
    target: "process",
    output: "<h2>Release &lt;notes&gt;</h2>\n",
  },
  {
    slug: "p",
    target: "process",
    output: "<p>Search &amp; navigation improvements</p>\n",
  },
  {
    slug: "text",
    target: "process",
    output: "<p>Updated &lt;today&gt;: <span>v0.61 &amp; later</span></p>\n",
  },
  {
    slug: "button",
    target: "process",
    output:
      '<button aria-expanded="false" type="button">Show details</button>\n<button aria-expanded="false" disabled type="button">Show details</button>\n',
  },
  {
    slug: "attribute",
    target: "process",
    output:
      '<section title="Read &quot;release notes&quot;" data-release="v1&quot;&amp;">&lt;script&gt; &amp; café</section>\nReservedAttributeName onclick\n',
  },
  {
    slug: "rendertostring",
    target: "process",
    output:
      '<p title="Read &quot;release notes&quot; &amp; café">&lt;script&gt; &amp; café</p>\n',
  },
  {
    slug: "renderdocument",
    target: "process",
    output: "<!doctype html><p>Release</p>\n",
  },
  {
    slug: "release-card-view",
    target: "process",
    output:
      '<section class="release-card"><h2>Release &lt;notes&gt;</h2><p hidden>Search &amp; navigation improvements</p><button id="toggle-details" aria-expanded="false" type="button">Show details</button></section>\n<section class="release-card"><h2>Release &lt;notes&gt;</h2><p>Search &amp; navigation improvements</p><button id="toggle-details" aria-expanded="true" type="button">Hide details</button></section>\n<section class="release-card"><h2>Release &lt;notes&gt;</h2><p hidden>Search &amp; navigation improvements</p><button id="toggle-details" aria-expanded="false" type="button">Show details</button></section>\n',
    typescript: "release-card-view",
  },
  {
    slug: "release-card-app",
    target: "web",
    typescript: "release-card-app",
  },
] as const

// Exact compiler-owned identities. These do not create any new site routes.
export const webReaderRoutes = [
  {
    identity: "std/web/html",
    namespace: "module",
    kind: "module",
    route: "/docs/library/web/html/",
    module: "std/web/html",
    example: "html-module",
    slug: "html-module",
  },
  {
    identity: "std/web/html::Html",
    name: "Html",
    module: "std/web/html",
    kind: "opaque-type",
    namespace: "type",
    route: "/docs/library/web/html/opaque-type/html/",
    example: "cards",
    slug: "html",
  },
  {
    identity: "std/web/html::ElementProps",
    name: "ElementProps",
    module: "std/web/html",
    kind: "alias",
    namespace: "type",
    route: "/docs/library/web/html/alias/elementprops/",
    example: "elementprops",
    slug: "elementprops",
  },
  {
    identity: "std/web/html::trait(IntoChildren)",
    name: "IntoChildren",
    module: "std/web/html",
    kind: "trait",
    namespace: "trait",
    route: "/docs/library/web/html/trait/intochildren/",
    example: "intochildren",
    slug: "intochildren",
  },
  {
    identity: "std/web/html::section",
    name: "section",
    module: "std/web/html",
    kind: "function",
    namespace: "value",
    route: "/docs/library/web/html/function/section/",
    example: "cards",
    slug: "section",
  },
  {
    identity: "std/web/html::h2",
    name: "h2",
    module: "std/web/html",
    kind: "function",
    namespace: "value",
    route: "/docs/library/web/html/function/h2/",
    example: "h2",
    slug: "h2",
  },
  {
    identity: "std/web/html::p",
    name: "p",
    module: "std/web/html",
    kind: "function",
    namespace: "value",
    route: "/docs/library/web/html/function/p/",
    example: "p",
    slug: "p",
  },
  {
    identity: "std/web/html::text",
    name: "text",
    module: "std/web/html",
    kind: "function",
    namespace: "value",
    route: "/docs/library/web/html/function/text/",
    example: "text",
    slug: "text",
  },
  {
    identity: "std/web/html::fragment",
    name: "fragment",
    module: "std/web/html",
    kind: "function",
    namespace: "value",
    route: "/docs/library/web/html/function/fragment/",
    example: "cards",
    slug: "fragment",
  },
  {
    identity: "std/web/html::button",
    name: "button",
    module: "std/web/html",
    kind: "function",
    namespace: "value",
    route: "/docs/library/web/html/function/button/",
    example: "button",
    slug: "button",
  },
  {
    identity: "std/web/html::attribute",
    name: "attribute",
    module: "std/web/html",
    kind: "function",
    namespace: "value",
    route: "/docs/library/web/html/function/attribute/",
    example: "attribute",
    slug: "attribute",
  },
  {
    identity: "std/web/html::renderToString",
    name: "renderToString",
    module: "std/web/html",
    kind: "function",
    namespace: "value",
    route: "/docs/library/web/html/function/rendertostring/",
    example: "rendertostring",
    slug: "rendertostring",
  },
  {
    identity: "std/web/html::renderDocument",
    name: "renderDocument",
    module: "std/web/html",
    kind: "function",
    namespace: "value",
    route: "/docs/library/web/html/function/renderdocument/",
    example: "renderdocument",
    slug: "renderdocument",
  },
  {
    identity: "std/web/dom",
    namespace: "module",
    kind: "module",
    route: "/docs/library/web/dom/",
    module: "std/web/dom",
    example: "release-card-app",
    slug: "dom-module",
  },
  {
    identity: "std/web/dom::app",
    name: "app",
    module: "std/web/dom",
    kind: "function",
    namespace: "value",
    route: "/docs/library/web/dom/function/app/",
    example: "release-card-app",
    slug: "app",
  },
] as const

export const webReaderFailures = [
  {
    slug: "wrong-hidden",
    diagnostics: ["SES-T0101"],
    correction: "elementprops",
  },
  {
    slug: "mixed-children",
    diagnostics: ["SES-T0101", "SES-T0201"],
    correction: "intochildren",
  },
  {
    slug: "unanchored-children",
    diagnostics: ["SES-T0201"],
    correction: "cards",
  },
] as const

export function webReaderExamples(playgroundUrl: string) {
  return webReaderCases.flatMap((item) => {
    const source = canonicalExample(
      `web-reader-${item.slug}`,
      `apps/site/examples/src/web-reader/${item.slug}.ssrg`,
      playgroundUrl,
      true
    )
    return "typescript" in item
      ? [
          source,
          canonicalExample(
            `web-reader-${item.slug}-ts`,
            `apps/site/examples/comparisons/web-reader/${item.typescript}.ts`,
            playgroundUrl,
            false,
            "typescript"
          ),
        ]
      : [source]
  })
}
