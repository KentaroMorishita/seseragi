import { expect, test } from "bun:test"
import { readdirSync, readFileSync } from "node:fs"
import { resolve } from "node:path"
import { compilerReferenceModules } from "../scripts/reference"
import { referenceDescription } from "../scripts/reference-copy"
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
    signatureReading("Just: A -> Maybe<A>", "constructor", ["A"], []).ja
  ).toContain("Aの順で渡してください")
  expect(
    signatureReading("Just: A -> Maybe<A>", "constructor", ["A"], []).ja
  ).not.toContain("Just: Aの順")
  expect(
    signatureReading("identity<A> value: A -> A", "function", ["A"], []).ja
  ).toContain("戻り値の型はA")
})

test("every compiler API has paired reader copy and a declaration walkthrough", () => {
  const modules = compilerReferenceModules()
  const items = modules.flatMap(({ items }) => items)
  const instanceCopy = readFileSync(
    resolve(import.meta.dir, "../src/reference/instance-copy.ssrg"),
    "utf8"
  )
  expect(modules).toHaveLength(63)
  expect(items).toHaveLength(1812)
  for (const item of items) {
    if (item.itemKind !== "instance") {
      expect(item.descriptionEn, item.identity).not.toBe("")
      expect(item.descriptionJa, item.identity).not.toBe("")
      expect(item.descriptionJa, item.identity).not.toBe(item.description)
      expect(referenceDescription(item.description), item.identity).toEqual({
        en: item.descriptionEn,
        ja: item.descriptionJa,
      })
    } else {
      const traitName = item.name.split("<")[0]
      expect(instanceCopy, item.identity).toContain(
        `"${traitName}" -> localized`
      )
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

test("unknown upstream descriptions fail instead of bypassing bilingual review", () => {
  expect(() => referenceDescription("Unreviewed new API description")).toThrow(
    "Missing bilingual API explanation"
  )
  expect(
    referenceDescription(
      "Standard function provided by the compiler-owned library surface."
    )
  ).toEqual({
    ja: "標準ライブラリの関数です。渡す引数と戻り値の型は、下の宣言とその読み方で確認してください。",
    en: "A standard-library function. See the declaration and its explanation below for the argument types and result type.",
  })
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
        /makeExplanation\s+\(\s*ArticleExample\s+"([^"]+)"\s*\)/u
      )?.[1]
      const group = guide.match(
        /makeExplanation\s+\(\s*ArticleExamples\s+\[([^\]]*)\]\s*\)/u
      )?.[1]
      if (group !== undefined) {
        const ids = [...group.matchAll(/"([^"]+)"/gu)].map((match) => match[1])
        expect(group.replace(/"[^"]+"|[\s,]/gu, ""), directory).toBe("")
        expect(ids.length, directory).toBeGreaterThan(0)
        expect(new Set(ids).size, directory).toBe(ids.length)
        const panels = [
          ...source.matchAll(/CodeExample\s*\(\s*"([^"]+)"/gu),
        ].map((match) => match[1])
        for (const selected of ids)
          expect(
            panels.filter((panel) => panel === selected),
            directory
          ).toHaveLength(1)
      } else if (id) expect(source, directory).toContain(`"${id}"`)
      else expect(guide, directory).toContain("makeExplanation WithoutExample")
      count++
    }
  expect(count).toBe(105)
  // This is structural regression coverage, not a claim that prose is good.
})
