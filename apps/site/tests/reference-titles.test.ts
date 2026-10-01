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

test("period-separated purpose links retain exact titles in both locales", () => {
  const englishDestination = "/docs/language/model/expression-oriented/"
  const localizedTitles = new Map([
    [englishDestination, "Expression-oriented"],
    [destination, "式指向"],
  ])
  for (const [path, title, separator] of [
    [
      englishDestination,
      "Expression-oriented",
      ". Read how branches return values.",
    ],
    [destination, "式指向", "。分岐が返す値を調べます。"],
  ]) {
    expect(
      assertReferenceLinkTitles(
        `<article><p><a href="${path}">${title}</a><span>${separator}</span></p></article>`,
        localizedTitles,
        route
      )
    ).toBe(1)
    expect(() =>
      assertReferenceLinkTitles(
        `<article><p><a href="${path}">Wrong title</a><span>${separator}</span></p></article>`,
        localizedTitles,
        route
      )
    ).toThrow("must match its heading")
  }
})

test("sentence links do not become title links merely because their paragraph has punctuation", () => {
  expect(
    assertReferenceLinkTitles(
      `<article>
      <p><span>まず</span><a href="${destination}">詳しく読む</a><span>。その後で戻ります。</span></p>
      <p><a href="${destination}">値を返す式</a><span>を調べてください。</span></p>
      <p><a href="${destination}">An ordinary sentence link</a><span>.without a separating space</span></p>
    </article>`,
      titles,
      route
    )
  ).toBe(0)
})

test("adding a purpose to list references preserves exact-title coverage", () => {
  for (const separator of [
    "：目的",
    ": Purpose",
    "— purpose",
    ". Purpose",
    "。目的",
  ]) {
    const html = `<article><ul>
      <li><a href="${destination}">式指向</a></li>
      <li><a href="${destination}">式指向</a><span>${separator}</span></li>
      </ul><p><a href="${destination}">式指向</a><span>${separator}</span></p></article>`
    expect(assertReferenceLinkTitles(html, titles, route)).toBe(3)
    expect(() =>
      assertReferenceLinkTitles(
        html.replace(
          `式指向</a><span>${separator}</span></li>`,
          `Wrong title</a><span>${separator}</span></li>`
        ),
        titles,
        route
      )
    ).toThrow("must match its heading")
  }
})

test("purpose-like list prose excludes fragments, external links and ordinary sentences", () => {
  expect(
    assertReferenceLinkTitles(
      `<article><ul>
    <li><a href="${destination}#rule">A section</a><span>. Purpose</span></li>
    <li><a href="https://example.com/">An external source</a><span>. Purpose</span></li>
    <li><span>First </span><a href="${destination}">read more</a><span>. Then return.</span></li>
    <li><a href="${destination}">An ordinary sentence</a><span> continues here.</span></li>
    <li><a href="${destination}">An ordinary sentence</a><span>.without a separating space</span></li>
    <li><a href="${destination}">An ordinary sentence</a><span>:without a separating space</span></li>
    </ul></article>`,
      titles,
      route
    )
  ).toBe(0)
})
