import { describe, it, expect, beforeEach, afterEach } from "bun:test"
import * as fs from "fs"
import * as path from "path"
import { compileCommand } from "../src/cli/compile"

const testDir = path.join(__dirname, "tmp")
const testInputFile = path.join(testDir, "test.ssrg")
const testOutputFile = path.join(testDir, "test.ts")

describe("Compile Command", () => {
  beforeEach(() => {
    // テスト用ディレクトリの作成
    if (!fs.existsSync(testDir)) {
      fs.mkdirSync(testDir, { recursive: true })
    }
  })

  afterEach(() => {
    // テスト用ファイルのクリーンアップ
    ;[testInputFile, testOutputFile].forEach((file) => {
      if (fs.existsSync(file)) {
        fs.unlinkSync(file)
      }
    })
  })

  it("should compile simple function to TypeScript", async () => {
    // テスト用のSeseragiソースコード
    const seseragiCode = `fn add a: Int -> b: Int -> Int = a + b`

    fs.writeFileSync(testInputFile, seseragiCode)

    await compileCommand({
      input: testInputFile,
      output: testOutputFile,
      generateComments: false,
      useArrowFunctions: true,
    })

    expect(fs.existsSync(testOutputFile)).toBe(true)

    const compiledCode = fs.readFileSync(testOutputFile, "utf-8")
    expect(compiledCode).toContain("add")
    expect(compiledCode).toContain("=>")
  })

  it("should compile variable declaration to TypeScript", async () => {
    const seseragiCode = `let greeting: String = "Hello, Seseragi!"`

    fs.writeFileSync(testInputFile, seseragiCode)

    await compileCommand({
      input: testInputFile,
      output: testOutputFile,
      generateComments: false,
      useArrowFunctions: true,
    })

    expect(fs.existsSync(testOutputFile)).toBe(true)

    const compiledCode = fs.readFileSync(testOutputFile, "utf-8")
    expect(compiledCode).toContain("greeting")
    expect(compiledCode).toContain("Hello, Seseragi!")
  })

  it("should use default output filename when not specified", async () => {
    const seseragiCode = `fn double x: Int -> Int = x * 2`

    fs.writeFileSync(testInputFile, seseragiCode)

    await compileCommand({
      input: testInputFile,
      generateComments: false,
      useArrowFunctions: true,
    })

    const defaultOutputFile = path.join(testDir, "test.ts")
    expect(fs.existsSync(defaultOutputFile)).toBe(true)

    const compiledCode = fs.readFileSync(defaultOutputFile, "utf-8")
    expect(compiledCode).toContain("double")
  })

  it("should throw error for non-existent input file", async () => {
    const nonExistentFile = path.join(testDir, "nonexistent.ssrg")

    await expect(
      compileCommand({
        input: nonExistentFile,
        output: testOutputFile,
      })
    ).rejects.toThrow("Input file not found")
  })

  it("should compile with function declarations when useArrowFunctions is false", async () => {
    const seseragiCode = `fn multiply x: Int -> y: Int -> Int = x * y`

    fs.writeFileSync(testInputFile, seseragiCode)

    await compileCommand({
      input: testInputFile,
      output: testOutputFile,
      generateComments: false,
      useArrowFunctions: false,
    })

    expect(fs.existsSync(testOutputFile)).toBe(true)

    const compiledCode = fs.readFileSync(testOutputFile, "utf-8")
    expect(compiledCode).toContain("function")
  })
})
