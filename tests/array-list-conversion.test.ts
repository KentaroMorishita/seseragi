import { describe, expect, test } from "bun:test"
import { lex } from "../src/lexer"
import { parse } from "../src/parser"
import { infer } from "../src/type-inference"
import { generateTypeScript } from "../src/codegen"
import * as fs from "node:fs"
import * as path from "node:path"
import { execSync } from "node:child_process"

describe("Array↔List Conversion", () => {
  const testCases = [
    {
      name: "arrayToList with integer array",
      code: `
let arr = [1, 2, 3, 4, 5]
let list = arrayToList arr

// listToArray で元に戻す
let arr2 = listToArray list

print arr
print list
print arr2
`,
      expectedOutput:
        '[ 1, 2, 3, 4, 5 ]\n{\n  tag: "Cons",\n  head: 1,\n  tail: {\n    tag: "Cons",\n    head: 2,\n    tail: {\n      tag: "Cons",\n      head: 3,\n      tail: [Object ...],\n    },\n  },\n}\n[ 1, 2, 3, 4, 5 ]\n',
    },
    {
      name: "List operations after arrayToList",
      code: `
let scores = [85, 92, 78]
let scoreList = arrayToList scores

print (listToArray scoreList)
`,
      expectedOutput: "[ 85, 92, 78 ]\n",
    },
    {
      name: "Empty array conversion",
      code: `
let empty = []
let emptyList = arrayToList empty
let backToArray = listToArray emptyList

print empty
print emptyList
print backToArray
`,
      expectedOutput: '[]\n{\n  tag: "Empty",\n}\n[]\n',
    },
    {
      name: "String array conversion",
      code: `
let words = ["hello", "world", "seseragi"]
let wordList = arrayToList words
let wordsBack = listToArray wordList

print words
print wordList
print wordsBack
`,
      expectedOutput:
        '[ "hello", "world", "seseragi" ]\n{\n  tag: "Cons",\n  head: "hello",\n  tail: {\n    tag: "Cons",\n    head: "world",\n    tail: {\n      tag: "Cons",\n      head: "seseragi",\n      tail: [Object ...],\n    },\n  },\n}\n[ "hello", "world", "seseragi" ]\n',
    },
    {
      name: "Mixed with backtick list syntax",
      code: `
// Arrayから変換
let arr = [1, 2, 3]
let list1 = arrayToList arr

// 直接リスト構文で作成
let list2 = \`[1, 2, 3]

// 両方を配列に変換して比較
let arr1 = listToArray list1
let arr2 = listToArray list2

print arr1
print arr2
`,
      expectedOutput: "[ 1, 2, 3 ]\n[ 1, 2, 3 ]\n",
    },
  ]

  for (const testCase of testCases) {
    test(testCase.name, () => {
      // Lexing
      const tokens = lex(testCase.code)

      // Parsing
      const parseResult = parse(tokens)
      expect(parseResult.errors).toEqual([])
      expect(parseResult.statements).toBeDefined()

      // Type inference
      const typeResult = infer(parseResult.statements!)
      expect(typeResult.errors).toEqual([])

      // Code generation
      const generated = generateTypeScript(parseResult.statements!, {
        runtimeMode: "embedded",
      })

      // 一時ファイルに書き込んで実行
      const tempFile = path.join(
        process.cwd(),
        `test-array-list-conv-${Date.now()}.ts`
      )
      fs.writeFileSync(tempFile, generated)

      try {
        // TypeScriptを実行
        const output = execSync(`bun run ${tempFile}`, {
          encoding: "utf-8",
        }).toString()

        expect(output).toBe(testCase.expectedOutput)
      } finally {
        // クリーンアップ
        fs.unlinkSync(tempFile)
      }
    })
  }

  test("Type checking for arrayToList and listToArray", () => {
    const code = `
// arrayToList の型推論テスト
let intArray = [1, 2, 3]
let intList = arrayToList intArray  // List<Int>

let strArray = ["a", "b", "c"]
let strList = arrayToList strArray  // List<String>

// listToArray の型推論テスト
let backToIntArray = listToArray intList    // Array<Int>
let backToStrArray = listToArray strList    // Array<String>
`

    const tokens = lex(code)
    const parseResult = parse(tokens)
    const typeResult = infer(parseResult.statements!)

    expect(typeResult.errors).toEqual([])
    expect(typeResult.inferredTypes).toBeDefined()

    // 型推論結果の確認
    const types = typeResult.inferredTypes!

    // intListの型はList<Int>であるべき
    const intListType = types.get("intList")
    expect(intListType).toBeDefined()
    expect(intListType?.kind).toBe("GenericType")

    // strListの型はList<String>であるべき
    const strListType = types.get("strList")
    expect(strListType).toBeDefined()
    expect(strListType?.kind).toBe("GenericType")

    // backToIntArrayの型はArray<Int>であるべき
    const backToIntArrayType = types.get("backToIntArray")
    expect(backToIntArrayType).toBeDefined()
    expect(backToIntArrayType?.kind).toBe("GenericType")

    // backToStrArrayの型はArray<String>であるべき
    const backToStrArrayType = types.get("backToStrArray")
    expect(backToStrArrayType).toBeDefined()
    expect(backToStrArrayType?.kind).toBe("GenericType")
  })
})
