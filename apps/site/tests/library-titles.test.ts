import { expect, test } from "bun:test"
import {
  assertAuthoredLibraryTitles,
  authoredLibraryArticle,
} from "./library-titles"

const module = { route: "/docs/library/example/", kind: "module" as const }
const api = {
  route: "/docs/library/example/function/get/",
  kind: "api" as const,
}
const titles = new Map([
  [module.route, "std/example"],
  [api.route, "get"],
])
const generated = `<h2 id="using-this-module">Using this module</h2><h2 id="public-api">Public API</h2><ul><li><a href="${api.route}">get · function</a></li></ul>`

test("authored module links retain exact titles while generated kind labels stay excluded", () => {
  const authored = `<h2 id="task">Choose a task</h2><li><a href="${api.route}">get</a><span>: read a value.</span></li>`
  const html = `<article>${authored}${generated}</article>`
  expect(authoredLibraryArticle(html, module)).toBe(
    `<article>${authored}</article>`
  )
  expect(assertAuthoredLibraryTitles(html, titles, module)).toBe(1)
  expect(() =>
    assertAuthoredLibraryTitles(
      html.replace(">get</a>", ">Wrong</a>"),
      titles,
      module
    )
  ).toThrow("must match its heading")
})

test("API footer titles and same-title links are checked without a generated-index exemption", () => {
  const html = `<article><p><a href="${module.route}">std/example</a><span>: choose another operation.</span></p></article>`
  expect(assertAuthoredLibraryTitles(html, titles, api)).toBe(1)
  expect(() =>
    assertAuthoredLibraryTitles(
      html.replace("std/example</a>", "Example</a>"),
      titles,
      api
    )
  ).toThrow("must match its heading")
  expect(() =>
    assertAuthoredLibraryTitles(`<article>${generated}</article>`, titles, api)
  ).toThrow("must match its heading")
})

test("navigation, fragment references, external links and descriptive standalone CTAs remain separate", () => {
  const html = `<nav><a href="${module.route}">Navigation</a></nav><article>
    <p><a href="${module.route}">Choose an operation for your task</a></p>
    <p><span>Read </span><a href="${module.route}">the options</a><span> before choosing.</span></p>
    <li><a href="${module.route}#task">Jump to task</a></li>
    <li><a href="https://example.invalid/">External</a></li>
  </article>`
  expect(assertAuthoredLibraryTitles(html, titles, api)).toBe(0)
})

test("missing articles, module boundaries, introductions and catalog titles fail explicitly", () => {
  expect(() => authoredLibraryArticle("<main></main>", api)).toThrow(
    "missing authored article"
  )
  expect(() =>
    authoredLibraryArticle("<article><p>Intro</p></article>", module)
  ).toThrow("missing generated module boundary")
  expect(() =>
    authoredLibraryArticle(`<article>${generated}</article>`, module)
  ).toThrow("empty authored module introduction")
  expect(() =>
    assertAuthoredLibraryTitles("<article></article>", new Map(), api)
  ).toThrow("missing catalog title")
})

test("Japanese page names and purpose remain distinct inside authored module introductions", () => {
  const entry = { route: "/ja/docs/library/example/", kind: "module" as const }
  const destination = "/ja/docs/library/example/function/get/"
  const localized = new Map([
    [entry.route, "std/example"],
    [destination, "get"],
  ])
  const html = `<article><li><a href="${destination}">get</a><span>：値を読みます。</span></li>${generated}</article>`
  expect(assertAuthoredLibraryTitles(html, localized, entry)).toBe(1)
  expect(() =>
    assertAuthoredLibraryTitles(
      html.replace(">get</a>", ">getで値を読む</a>"),
      localized,
      entry
    )
  ).toThrow("must match its heading")
})
