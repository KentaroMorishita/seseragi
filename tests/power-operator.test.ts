import { describe, it, expect, beforeEach, afterEach } from "bun:test"
import * as fs from "fs"
import * as path from "path"
import { compileCommand } from "../src/cli/compile"

const testDir = path.join(__dirname, "tmp")
const testInputFile = path.join(testDir, "power.ssrg")
const testOutputFile = path.join(testDir, "power.ts")

describe("Power operator (**)", () => {
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

  it("should compile basic power operation", async () => {
    const seseragiCode = `
      let base: Int = 2
      let exp: Int = 3
      let result = base ** exp
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
    expect(compiledCode).toContain("(base ** exp)")
  })

  it("should handle float power operations", async () => {
    const seseragiCode = `
      let x: Float = 2.5
      let y: Float = 2.0
      let result = x ** y
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
    expect(compiledCode).toContain("(x ** y)")
  })

  it("should handle right associativity", async () => {
    const seseragiCode = `
      let a: Int = 2
      let b: Int = 3
      let c: Int = 2
      let result = a ** b ** c
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
    expect(compiledCode).toContain("(a ** (b ** c))")
  })

  it("should have correct precedence with multiplication", async () => {
    const seseragiCode = `
      let a: Int = 2
      let b: Int = 3
      let c: Int = 4
      let result = a * b ** c
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
    expect(compiledCode).toContain("(a * (b ** c))")
  })

  it("should work with negative numbers", async () => {
    const seseragiCode = `
      let base: Int = -2
      let exp: Int = 3
      let result = base ** exp
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
    expect(compiledCode).toContain("**")
  })

  it("should work in function expressions", async () => {
    const seseragiCode = `
      fn power base: Int -> exp: Int -> Int = base ** exp
      let result = power 2 8
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
    expect(compiledCode).toContain("**")
  })

  it("should work with parentheses", async () => {
    const seseragiCode = `
      let a: Int = 2
      let b: Int = 3
      let c: Int = 4
      let result = (a + b) ** c
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
    expect(compiledCode).toContain("((a + b) ** c)")
  })
})
