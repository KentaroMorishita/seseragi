import { expect, test } from "bun:test"
import { plannedReferenceRoutes, referenceCoverage } from "../scripts/coverage"

const complete = plannedReferenceRoutes.flatMap((route) => [
  route,
  `/ja${route}`,
])

test("coverage includes the grammar appendix and package reference", () => {
  const coverage = referenceCoverage(complete)
  expect(coverage.find(({ area }) => area === "language")?.planned).toBe(106)
  expect(coverage.find(({ area }) => area === "packages")?.planned).toBe(11)
  expect(coverage.every(({ missing }) => missing.length === 0)).toBe(true)
})

test("coverage rejects a missing leaf, locale or unplanned language route", () => {
  expect(() =>
    referenceCoverage(
      complete.filter((route) => route !== "/docs/language/grammar/")
    )
  ).toThrow("Missing core reference article")
  expect(() =>
    referenceCoverage(
      complete.filter((route) => route !== "/ja/docs/language/grammar/")
    )
  ).toThrow("Missing Japanese")
  expect(() =>
    referenceCoverage([...complete, "/docs/language/unknown/"])
  ).toThrow("Unmapped language article")
})
