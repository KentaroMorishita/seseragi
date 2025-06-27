import { describe, expect, test } from "bun:test"
import { lex } from "../src/lexer"
import { parse } from "../src/parser"
import { infer } from "../src/type-inference"
import { generateTypeScript } from "../src/codegen"
import * as fs from "fs"
import * as path from "path"
import { execSync } from "child_process"

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
      expectedOutput: "[ 1, 2, 3, 4, 5 ]\n{ tag: 'Cons', head: 1, tail: { tag: 'Cons', head: 2, tail: { tag: 'Cons', head: 3, tail: { tag: 'Cons', head: 4, tail: { tag: 'Cons', head: 5, tail: { tag: 'Empty' } } } } } }\n[ 1, 2, 3, 4, 5 ]\n",
    },
    {
      name: "List operations after arrayToList",
      code: `
let scores = [85, 92, 78]
let scoreList = arrayToList scores

// List関数を使用（mapは手動実装）
fn mapList f lst =
  match lst {
    Empty -> Empty
    Cons x xs -> Cons (f x) (mapList f xs)
  }

let add10 = fn x -> x + 10
let updatedList = mapList add10 scoreList
let result = listToArray updatedList

print result
`,
      expectedOutput: "[ 95, 102, 88 ]\n",
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
      expectedOutput: "[]\n{ tag: 'Empty' }\n[]\n",
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
      expectedOutput: '[ "hello", "world", "seseragi" ]\n{ tag: \'Cons\', head: "hello", tail: { tag: \'Cons\', head: "world", tail: { tag: \'Cons\', head: "seseragi", tail: { tag: \'Empty\' } } } }\n[ "hello", "world", "seseragi" ]\n',
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
    expect(intListType?.toString()).toContain("List")
    
    // strListの型はList<String>であるべき
    const strListType = types.get("strList")
    expect(strListType).toBeDefined()
    expect(strListType?.toString()).toContain("List")
    
    // backToIntArrayの型はArray<Int>であるべき
    const backToIntArrayType = types.get("backToIntArray")
    expect(backToIntArrayType).toBeDefined()
    expect(backToIntArrayType?.toString()).toContain("Array")
    
    // backToStrArrayの型はArray<String>であるべき
    const backToStrArrayType = types.get("backToStrArray")
    expect(backToStrArrayType).toBeDefined()
    expect(backToStrArrayType?.toString()).toContain("Array")
  })
})