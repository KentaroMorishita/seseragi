import { expect, test } from "bun:test"
import { assertReferenceLinkTitles, pageTitle } from "./reference-titles"

const route = "/ja/docs/language/model/design-principles/"
const destination = "/ja/docs/language/model/expression-oriented/"
const titles = new Map([[destination, "式指向"]])

test("page-name links must match the localized destination title", () => {
  expect(pageTitle("<h1><span>式指向</span></h1>")).toBe("式指向")
  expect(
    assertReferenceLinkTitles(
      `<article><ul><li><a href="${destination}">式指向</a></li></ul></article>`,
      titles,
      route
    )
  ).toBe(1)
  expect(() =>
    assertReferenceLinkTitles(
      `<article><ul><li><a href="${destination}">式中心</a></li></ul></article>`,
      titles,
      route
    )
  ).toThrow("must match its heading")
})

test("descriptive prose, section anchors and navigation are not page names", () => {
  expect(
    assertReferenceLinkTitles(
      `<nav><a href="${destination}">ナビゲーション</a></nav>
       <article><p><a href="${destination}">詳しく読む</a></p>
       <ul><li><a href="${destination}#core-rule">制御構造も式</a></li>
       <li><a href="${destination}">説明</a>を本文で続ける</li></ul></article>`,
      titles,
      route
    )
  ).toBe(0)
})

test("encoded headings and titles compare as visible text", () => {
  const encoded = new Map([["/docs/language/test/", "A & B"]])
  expect(pageTitle("<h1>A &amp; B</h1>")).toBe("A & B")
  expect(
    assertReferenceLinkTitles(
      '<article><li><a href="/docs/language/test/"><span>A &amp; B</span></a></li></article>',
      encoded,
      "/docs/language/"
    )
  ).toBe(1)
})

test("purpose-labelled references retain the page title", () => {
  expect(
    assertReferenceLinkTitles(
      `<article><p><a href="${destination}">式指向</a><span>：値を返す制御構造を確認できます。</span></p></article>`,
      titles,
      route
    )
  ).toBe(1)
  expect(() =>
    assertReferenceLinkTitles(
      `<article><p><a href="${destination}">式中心</a><span>：値を返す制御構造を確認できます。</span></p></article>`,
      titles,
      route
    )
  ).toThrow("must match its heading")
})
