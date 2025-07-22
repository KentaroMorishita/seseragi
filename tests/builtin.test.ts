import { describe, it, expect, beforeEach, afterEach } from "bun:test"
import * as fs from "node:fs"
import * as path from "node:path"
import { compileCommand } from "../src/cli/compile"

const testDir = path.join(__dirname, "tmp")
const testInputFile = path.join(testDir, "builtin.ssrg")
const testOutputFile = path.join(testDir, "builtin.ts")

describe("Builtin Functions", () => {
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

  it("should compile print function to console.log", async () => {
    const seseragiCode = `print("Hello, World!")`

    fs.writeFileSync(testInputFile, seseragiCode)

    await compileCommand({
      input: testInputFile,
      output: testOutputFile,
      generateComments: false,
      useArrowFunctions: true,
    })

    expect(fs.existsSync(testOutputFile)).toBe(true)

    const compiledCode = fs.readFileSync(testOutputFile, "utf-8")
    expect(compiledCode).toContain('ssrgPrint("Hello, World!")')
  })

  it("should compile putStrLn function to console.log", async () => {
    const seseragiCode = `putStrLn("Hello, Seseragi!")`

    fs.writeFileSync(testInputFile, seseragiCode)

    await compileCommand({
      input: testInputFile,
      output: testOutputFile,
      generateComments: false,
      useArrowFunctions: true,
    })

    const compiledCode = fs.readFileSync(testOutputFile, "utf-8")
    expect(compiledCode).toContain('ssrgPutStrLn("Hello, Seseragi!")')
  })

  it("should compile toString function to toString()", async () => {
    const seseragiCode = `
    let age: Int = 25
    print(toString(age))
    `

    fs.writeFileSync(testInputFile, seseragiCode)

    await compileCommand({
      input: testInputFile,
      output: testOutputFile,
      generateComments: false,
      useArrowFunctions: true,
    })

    const compiledCode = fs.readFileSync(testOutputFile, "utf-8")
    expect(compiledCode).toContain("ssrgToString(age)")
    expect(compiledCode).toContain("ssrgPrint(ssrgToString(age))")
  })

  it("should compile multiple builtin functions", async () => {
    const seseragiCode = `
    let name: String = "Alice"
    let age: Int = 30
    print("Name: " + name)
    putStrLn(toString(age))
    `

    fs.writeFileSync(testInputFile, seseragiCode)

    await compileCommand({
      input: testInputFile,
      output: testOutputFile,
      generateComments: false,
      useArrowFunctions: true,
    })

    const compiledCode = fs.readFileSync(testOutputFile, "utf-8")
    expect(compiledCode).toContain('ssrgPrint(("Name: " + name))')
    expect(compiledCode).toContain("ssrgPutStrLn(ssrgToString(age))")
  })

  it("should handle builtin functions in function definitions", async () => {
    const seseragiCode = `
    fn greet name: String -> Unit = print("Hello, " + name)
    `

    fs.writeFileSync(testInputFile, seseragiCode)

    await compileCommand({
      input: testInputFile,
      output: testOutputFile,
      generateComments: false,
      useArrowFunctions: true,
    })

    const compiledCode = fs.readFileSync(testOutputFile, "utf-8")
    expect(compiledCode).toContain('ssrgPrint(("Hello, " + name))')
  })
})
