import { describe, expect, test } from "bun:test"
import { tourLessons } from "../src/tour/curriculum"
import { loadBindings, type ProjectResponse } from "./wasm-compiler-support"

describe("canonical Tour compiler gate", () => {
  for (const lesson of tourLessons) {
    for (const [kind, source] of [
      ["lesson", lesson.source],
      ["exercise", lesson.exerciseSource],
    ] as const) {
      if (source.trim() === "") continue
      test(`compiles ${lesson.id} ${kind}`, async () => {
        const wasm = await loadBindings()
        const compiled = JSON.parse(
          wasm.compile_single_file("main.ssrg", `tour/${lesson.id}`, source)
        ) as ProjectResponse
        expect(compiled.status).toBe("success")
        if (compiled.status !== "success") {
          throw new Error(
            `Tour ${lesson.id} ${kind} failed: ${JSON.stringify(compiled)}`
          )
        }
      })
    }
  }
})
