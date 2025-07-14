import { describe, test, expect } from "bun:test"
import { compileSeseragi } from "./test-utils"

describe("Template Function Integration Tests", () => {
  test("should compile show with template literal", () => {
    const source = "show `Hello World`"

    const result = compileSeseragi(source)
    expect(typeof result).toBe("string")
    expect(result).toContain("show(`Hello World`)")
  })

  test("should compile print with template literal and expression", () => {
    const source = "print `Count: ${42}`"

    const result = compileSeseragi(source)
    expect(typeof result).toBe("string")
    expect(result).toContain("console.log(`Count: ${42}`)")
  })
})
