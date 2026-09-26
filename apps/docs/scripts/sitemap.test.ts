import { describe, expect, test } from "bun:test"
import { resolve } from "node:path"
import {
  collectCoverageMap,
  collectSpecSections,
  validateRepositorySitemap,
  validateSitemap,
} from "./sitemap"

const sections = collectSpecSections([
  {
    file: "docs/spec/00-language.md",
    source: "# Chapter\n\n## 0.1 First\n\n### 0.1.1 Detail\n",
  },
])

describe("documentation sitemap coverage", () => {
  test("accepts a unique leaf route for every numbered semantic section", () => {
    const coverage = collectCoverageMap(`
| Route | Source | Scope |
| --- | --- | --- |
| \`/language/model/first/\` | 0.1 | First rule. |
| \`/language/model/detail/\` | 0.1.1 | Detailed rule. |

## Grammar Appendix

\`/language/grammar/\` is a navigable grammar reference.
`)
    expect(
      validateSitemap(sections, coverage, "# Appendix A. Grammar\n")
    ).toEqual({
      specFiles: 1,
      specSections: 2,
      routes: 3,
      sourceIds: 2,
    })
  })

  test("rejects missing sections, unknown source ids and duplicate routes", () => {
    const coverage = collectCoverageMap(`
| Route | Source | Scope |
| --- | --- | --- |
| \`/language/model/first/\` | 0.1 | First rule. |
| \`/language/model/first/\` | 0.2 | Unknown rule. |

\`/language/grammar/\` is a navigable grammar reference.
`)
    expect(() =>
      validateSitemap(sections, coverage, "# Appendix A. Grammar\n")
    ).toThrow("Duplicate canonical route")
    expect(() =>
      validateSitemap(sections, coverage, "# Appendix A. Grammar\n")
    ).toThrow("Unmapped specification section 0.1.1")
    expect(() =>
      validateSitemap(sections, coverage, "# Appendix A. Grammar\n")
    ).toThrow("Unknown specification section 0.2")
  })

  test("rejects invalid route spelling and duplicate spec identities", () => {
    expect(() =>
      collectSpecSections([
        { file: "a.md", source: "## 0.1 First" },
        { file: "b.md", source: "## 0.1 Again" },
      ])
    ).toThrow("Duplicate specification section 0.1")

    const coverage = collectCoverageMap(`
| \`/Language/Bad_Path/\` | 0.1 | Invalid route. |
\`/language/grammar/\` is a navigable grammar reference.
`)
    expect(() =>
      validateSitemap(sections.slice(0, 1), coverage, "# Appendix A. Grammar\n")
    ).toThrow("Invalid canonical route")
  })

  test("the checked-in map covers every normative chapter", () => {
    const report = validateRepositorySitemap(
      resolve(import.meta.dir, "../../..")
    )
    expect(report.specFiles).toBe(18)
    expect(report.specSections).toBeGreaterThan(290)
    expect(report.routes).toBeGreaterThan(300)
    expect(report.sourceIds).toBe(report.specSections)
  })
})
