/**
 * Tests for the show builtin function
 */

import { describe, it, expect, beforeEach, afterEach } from "bun:test"
import * as fs from "node:fs"
import * as path from "node:path"
import { compileCommand } from "../src/cli/compile"

const testDir = path.join(__dirname, "tmp")
const testInputFile = path.join(testDir, "show-test.ssrg")
const testOutputFile = path.join(testDir, "show-test.ts")

describe("Show Function", () => {
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

  it("should compile show function with integer", async () => {
    const seseragiCode = `show(42)`

    fs.writeFileSync(testInputFile, seseragiCode)

    await compileCommand({
      input: testInputFile,
      output: testOutputFile,
      generateComments: false,
      useArrowFunctions: true,
    })

    expect(fs.existsSync(testOutputFile)).toBe(true)

    const compiledCode = fs.readFileSync(testOutputFile, "utf-8")
    expect(compiledCode).toContain("ssrgShow(42)")
  })

  it("should compile show function with string", async () => {
    const seseragiCode = `show("Hello World")`

    fs.writeFileSync(testInputFile, seseragiCode)

    await compileCommand({
      input: testInputFile,
      output: testOutputFile,
      generateComments: false,
      useArrowFunctions: true,
    })

    expect(fs.existsSync(testOutputFile)).toBe(true)

    const compiledCode = fs.readFileSync(testOutputFile, "utf-8")
    expect(compiledCode).toContain('ssrgShow("Hello World")')
  })

  it("should include show function definition in generated code", async () => {
    const seseragiCode = `
let x = 42
show x
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

    // show関数の定義が含まれることを確認
    expect(compiledCode).toContain("function ssrgShow(value: unknown): void {")
    // show関数の呼び出しが含まれることを確認
    expect(compiledCode).toContain("ssrgShow(x)")
  })
})
