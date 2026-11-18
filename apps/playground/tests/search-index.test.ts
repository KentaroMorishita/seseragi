import { describe, expect, test } from "bun:test"
import curriculum from "../../../examples/tour/curriculum.json"
import type { AnalysisReferenceItem } from "../src/compiler/types"
import { discoverGroups, samples } from "../src/samples"
import {
  apiEntries,
  exampleEntries,
  lessonEntries,
  searchFilters,
  searchSections,
} from "../src/ui/search-index"

const examples = exampleEntries(
  samples.filter(({ kind }) => kind !== "lesson"),
  discoverGroups
)
const lessons = lessonEntries(curriculum)
const api = apiEntries([
  reference("map", "std/array", "fn map<A, B> f: (A -> B) -> Array<A>"),
  reference("println", "Prelude", "effect fn println value: String"),
])

describe("Search index", () => {
  test("offers the five scopes as English UI labels", () => {
    expect(searchFilters.map(({ label }) => label)).toEqual([
      "All",
      "Examples",
      "Lessons",
      "API",
      "Settings",
    ])
  })

  test("lists every example once, in group order, under its group title", () => {
    const grouped = discoverGroups.flatMap((group) => group.samples)
    expect(examples.map(({ key }) => key.slice("example:".length))).toEqual(
      grouped
    )
    const sections = searchSections(examples, "", "examples")
    expect(sections.map(({ title }) => title)).toEqual(
      discoverGroups.map(({ title }) => title)
    )
  })

  test("keeps API out of All until something is typed", () => {
    const entries = [...examples, ...lessons, ...api]
    const idle = searchSections(entries, "", "all")
    expect(idle.map(({ id }) => id)).toEqual(["examples", "lessons"])
    for (const section of idle) expect(section.entries.length).toBeLessThan(6)

    const typed = searchSections(entries, "println", "all")
    expect(typed.map(({ id }) => id)).toContain("api")
  })

  test("ranks a title match ahead of a body match and keeps every term", () => {
    const sections = searchSections(api, "map", "api")
    expect(sections[0]?.entries[0]?.title).toBe("map")
    expect(searchSections(api, "map println", "api")).toEqual([])
  })

  test("matches full-width and case variations", () => {
    const sections = searchSections(api, "ＰＲＩＮＴＬＮ", "api")
    expect(sections[0]?.entries[0]?.title).toBe("println")
  })

  test("names lessons by part and chapter", () => {
    const first = lessons[0]
    expect(first?.key).toMatch(/^lesson:/u)
    expect(first?.detail).toContain(" · ")
  })
})

function reference(
  name: string,
  module: string,
  signature: string
): AnalysisReferenceItem {
  return {
    name,
    module,
    signature,
    category: module === "Prelude" ? "Prelude" : "Array",
    kind: "function",
    description: `${name} の説明`,
    constraints: [],
  } as unknown as AnalysisReferenceItem
}
