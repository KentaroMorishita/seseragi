import { describe, it, expect, beforeEach, afterEach } from "bun:test"
import * as fs from "node:fs"
import * as path from "node:path"
import { compileCommand } from "../src/cli/compile"

const testDir = path.join(__dirname, "tmp")
const testInputFile = path.join(testDir, "division.ssrg")
const testOutputFile = path.join(testDir, "division.ts")

describe("Integer division", () => {
  beforeEach(() => {
    if (!fs.existsSync(testDir)) {
      fs.mkdirSync(testDir, { recursive: true })
    }
  })

  afterEach(() => {
    ;[testInputFile, testOutputFile].forEach((file) => {
      if (fs.existsSync(file)) {
        fs.unlinkSync(file)
      }
    })
  })

  it("should generate Math.trunc for Int/Int division", async () => {
    const seseragiCode = `
      let x: Int = 7
      let y: Int = 2
      let result = x / y
      print result
    `

    fs.writeFileSync(testInputFile, seseragiCode)

    await compileCommand({
      input: testInputFile,
      output: testOutputFile,
      generateComments: false,
      useArrowFunctions: true,
    })

    expect(fs.existsSync(testOutputFile)).toBe(true)

    const compiledCode = fs.readFileSync(testOutputFile, "utf-8")
    expect(compiledCode).toContain("Math.trunc(")
    expect(compiledCode).toContain("Math.trunc(x / y)")
  })

  it("should not use Math.trunc for Float division", async () => {
    const seseragiCode = `
      let x: Float = 7.0
      let y: Float = 2.0
      let result = x / y
      print result
    `

    fs.writeFileSync(testInputFile, seseragiCode)

    await compileCommand({
      input: testInputFile,
      output: testOutputFile,
      generateComments: false,
      useArrowFunctions: true,
    })

    expect(fs.existsSync(testOutputFile)).toBe(true)

    const compiledCode = fs.readFileSync(testOutputFile, "utf-8")
    // Float除算は Math.trunc を使わない（toInt関数は除く）
    expect(compiledCode).toContain("(x / y)")
    expect(compiledCode).not.toContain("Math.trunc(x / y)")
  })

  it("should handle negative integers correctly", async () => {
    const seseragiCode = `
      let a: Int = -7
      let b: Int = 2
      let result = a / b
      print result
    `

    fs.writeFileSync(testInputFile, seseragiCode)

    await compileCommand({
      input: testInputFile,
      output: testOutputFile,
      generateComments: false,
      useArrowFunctions: true,
    })

    expect(fs.existsSync(testOutputFile)).toBe(true)

    const compiledCode = fs.readFileSync(testOutputFile, "utf-8")
    expect(compiledCode).toContain("Math.trunc(")
  })

  it("should preserve other arithmetic operations", async () => {
    const seseragiCode = `
      let a: Int = 10
      let b: Int = 3
      let sum = a + b
      let diff = a - b
      let product = a * b
      let quotient = a / b
      let remainder = a % b
      print sum
      print diff
      print product
      print quotient
      print remainder
    `

    fs.writeFileSync(testInputFile, seseragiCode)

    await compileCommand({
      input: testInputFile,
      output: testOutputFile,
      generateComments: false,
      useArrowFunctions: true,
    })

    expect(fs.existsSync(testOutputFile)).toBe(true)

    const compiledCode = fs.readFileSync(testOutputFile, "utf-8")

    // Only division should use Math.trunc
    expect(compiledCode).toContain("(a + b)")
    expect(compiledCode).toContain("(a - b)")
    expect(compiledCode).toContain("(a * b)")
    expect(compiledCode).toContain("Math.trunc(a / b)")
    expect(compiledCode).toContain("(a % b)")
  })

  it("should work with function division", async () => {
    const seseragiCode = `
      fn divide a: Int -> b: Int -> Int = a / b
      let result = divide 7 2
      print result
    `

    fs.writeFileSync(testInputFile, seseragiCode)

    await compileCommand({
      input: testInputFile,
      output: testOutputFile,
      generateComments: false,
      useArrowFunctions: true,
    })

    expect(fs.existsSync(testOutputFile)).toBe(true)

    const compiledCode = fs.readFileSync(testOutputFile, "utf-8")
    expect(compiledCode).toContain("Math.trunc(")
  })
})
