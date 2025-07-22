import { describe, it, expect, beforeEach, afterEach } from "bun:test"
import * as fs from "node:fs"
import * as path from "node:path"
import { compileCommand } from "../src/cli/compile"

const testDir = path.join(__dirname, "tmp")
const testInputFile = path.join(testDir, "cast.ssrg")
const testOutputFile = path.join(testDir, "cast.ts")

describe("Cast functions (toInt/toFloat)", () => {
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

  it("should compile toInt function call", async () => {
    const seseragiCode = `
      let x: Float = 3.14
      let y = toInt x
      print y
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
    expect(compiledCode).toContain("toInt(")
    expect(compiledCode).toContain("Math.trunc(value)")
  })

  it("should compile toFloat function call", async () => {
    const seseragiCode = `
      let x: Int = 42
      let y = toFloat x
      print y
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
    expect(compiledCode).toContain("toFloat(")
    expect(compiledCode).toContain("function toFloat(value: any): number")
  })

  it("should work with string conversion", async () => {
    const seseragiCode = `
      let s: String = "42"
      let i = toInt s
      let f = toFloat "3.14"
      print i
      print f
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
    expect(compiledCode).toContain("toInt(")
    expect(compiledCode).toContain("toFloat(")
  })

  it("should work with pipe operator", async () => {
    const seseragiCode = `
      let result1 = 3.14 | toInt
      let result2 = 42 | toFloat
      print result1
      print result2
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
    expect(compiledCode).toContain("toInt")
    expect(compiledCode).toContain("toFloat")
  })
})
