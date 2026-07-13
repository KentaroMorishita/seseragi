import { describe, expect, test } from "bun:test"
import { classifyIdentifier } from "../src/editor/seseragi-language"

describe("Seseragi syntax classification", () => {
  test("classifies language declarations and Effect keywords", () => {
    for (const keyword of [
      "pub",
      "type",
      "fn",
      "effect",
      "do",
      "with",
      "fails",
    ]) {
      expect(classifyIdentifier(keyword)).toBe("keyword")
    }
  })

  test("distinguishes constructors, builtins, booleans, and values", () => {
    expect(classifyIdentifier("Effect")).toBe("builtin-type")
    expect(classifyIdentifier("Player1Wins")).toBe("type-name")
    expect(classifyIdentifier("True")).toBe("bool")
    expect(classifyIdentifier("decide")).toBe("variable")
  })
})
