import { expect, test } from "bun:test"
import { readdirSync, readFileSync } from "node:fs"
import { resolve } from "node:path"
import { compilerReferenceModules } from "../scripts/reference"
import { outerArrows, signatureReading } from "../scripts/signature-reading"

test("outer signature arrows preserve callback, record, and nested generic types", () => {
  expect(
    outerArrows("map<A, B> arg1: (A -> B) -> arg2: Array<A> -> Array<B>")
  ).toEqual(["map<A, B> arg1: (A -> B)", "arg2: Array<A>", "Array<B>"])
  expect(
    outerArrows(
      "subscribe arg1: (A -> Effect<{ clock: Clock }, Never, Unit>) -> arg2: Signal<A> -> Task<Subscription>"
    )
  ).toHaveLength(3)
  expect(
    outerArrows(
      "create arg1: { callback: (Int -> Int), values: Array<Maybe<Int>> } -> Result<Int>"
    )
  ).toHaveLength(2)
  expect(
    signatureReading("Just: A -> Maybe<A>", "constructor", ["A"], []).ja
  ).toContain("引数は1個")
  expect(
    signatureReading("identity<A> value: A -> A", "function", ["A"], []).ja
  ).toContain("戻り値の型はA")
})

test("every compiler API has a Japanese explanation and a declaration walkthrough", () => {
  const modules = compilerReferenceModules()
  const items = modules.flatMap(({ items }) => items)
  expect(modules).toHaveLength(63)
  expect(items).toHaveLength(1812)
  for (const item of items) {
    if (item.itemKind !== "instance") {
      expect(item.descriptionJa, item.identity).not.toBe("")
      expect(item.descriptionJa, item.identity).not.toBe(item.description)
    }
    expect(item.reading.en, item.identity).not.toBe("")
    expect(item.reading.ja, item.identity).not.toBe("")
    expect(item.reading.ja, item.identity).not.toMatch(
      /dictionary|operator ABI|整備中|#[0-9]+/u
    )
  }
  const moduleCopy = readFileSync(
    resolve(import.meta.dir, "../src/reference/module-copy.ssrg"),
    "utf8"
  )
  for (const module of modules)
    expect(moduleCopy).toContain(`"${module.specifier}" -> localized`)
})

test("every conceptual article owns a bilingual example explanation", () => {
  const root = resolve(import.meta.dir, "../src/pages/language")
  let count = 0
  for (const category of readdirSync(root, { withFileTypes: true }).filter(
    (d) => d.isDirectory()
  ))
    for (const page of readdirSync(resolve(root, category.name), {
      withFileTypes: true,
    }).filter((d) => d.isDirectory())) {
      const directory = resolve(root, category.name, page.name)
      if (page.name === "overview") continue
      const guide = readFileSync(resolve(directory, "guide.ssrg"), "utf8")
      const source = readFileSync(resolve(directory, "page.ssrg"), "utf8")
      expect(source, directory).toContain(
        "explainPage (guide.explanation ()) (content ())"
      )
      expect(guide, directory).toContain("(en.explanation ())")
      expect(guide, directory).toContain("(ja.explanation ())")
      expect(guide, directory).not.toContain("localized")
      for (const locale of ["en", "ja"]) {
        const copy = readFileSync(resolve(directory, `${locale}.ssrg`), "utf8")
        expect(copy, directory).toContain(
          "pub fn explanation -> ExplanationCopy"
        )
        expect(copy, directory).toContain("summary: (explanation ()).summary")
        for (const field of ["question", "summary", "reading", "result"])
          expect(copy, directory).toMatch(new RegExp(`  ${field}: "[^"]`))
      }
      const id = guide.match(
        /makeExplanation\s+\(ArticleExample "([^"]+)"\)/u
      )?.[1]
      if (id) expect(source, directory).toContain(`"${id}"`)
      else expect(guide, directory).toContain("makeExplanation WithoutExample")
      count++
    }
  expect(count).toBe(105)
  // This is structural regression coverage, not a claim that prose is good.
})
