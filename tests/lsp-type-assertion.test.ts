import { describe, expect, test } from "bun:test"
import * as AST from "../src/ast"
import { Parser } from "../src/parser"
import { infer, type InferResult } from "../src/inference/engine/infer"

// LSPサーバーから同じ関数をコピーしてテスト用に使用
interface SymbolInfo {
  type: string
  name: string
  finalType: AST.Type
  hasExplicitType?: boolean
}

function findSymbolWithEnhancedInference(
  program: AST.Program,
  symbol: string,
  inferenceResult: InferResult,
  offset: number,
  _text: string
): SymbolInfo | null {
  // 戻り値も複雑なので anyを維持
  console.log(`Searching for symbol: "${symbol}" at offset ${offset}`)

  if (!program.statements) {
    return null
  }

  for (const statement of program.statements) {
    if (
      statement.kind === "VariableDeclaration" &&
      (statement as any).name === symbol
    ) {
      console.log(`=== Looking for variable ${symbol} ===`)
      console.log(
        `Statement type in nodeTypeMap: ${inferenceResult.nodeTypeMap.has(statement)}`
      )
      console.log(
        `Initializer type in nodeTypeMap: ${inferenceResult.nodeTypeMap.has((statement as any).initializer)}`
      )

      // Log the actual type from nodeTypeMap
      const nodeType = inferenceResult.nodeTypeMap.get(statement)
      if (nodeType) {
        console.log(
          `NodeTypeMap type for statement: ${JSON.stringify(nodeType, null, 2)}`
        )
      }

      const initType = inferenceResult.nodeTypeMap.get(
        (statement as any).initializer
      )
      if (initType) {
        console.log(
          `NodeTypeMap type for initializer: ${JSON.stringify(initType, null, 2)}`
        )
      }

      // Use the enhanced node type mapping to get the resolved type
      let finalType = inferenceResult.nodeTypeMap.get(statement)

      if (!finalType) {
        // Fallback: look for the type in the initializer
        finalType = inferenceResult.nodeTypeMap.get(
          (statement as any).initializer
        )
        console.log(
          `Using initializer type: ${finalType ? finalType.kind : "none"}`
        )
      }

      // IMPORTANT: Always apply substitution to resolve type variables
      if (
        finalType &&
        inferenceResult.substitution &&
        inferenceResult.substitution.apply
      ) {
        const originalType = finalType
        finalType = inferenceResult.substitution.apply(finalType)
        console.log(
          `Applied substitution: ${JSON.stringify(originalType, null, 2)} -> ${JSON.stringify(finalType, null, 2)}`
        )
      }

      console.log(
        `Final resolved type for ${symbol}: ${JSON.stringify(finalType, null, 2)}`
      )

      if (finalType) {
        return {
          type: "variable",
          name: symbol,
          finalType: finalType,
          hasExplicitType: !!(statement as any).type,
        }
      }
    }
  }

  return null
}

function formatInferredTypeInfo(
  symbol: string,
  symbolInfo: SymbolInfo | null
): string {
  // シンボル情報の型が複雑なのでanyを維持
  if (!symbolInfo || !symbolInfo.finalType) {
    return `**${symbol}**: unknown`
  }

  const formatType = (type: AST.Type): string => {
    // 型オブジェクトの型が複雑なのでanyを維持
    if (!type) return "unknown"

    switch (type.kind) {
      case "PrimitiveType":
        return type.name
      case "FunctionType": {
        const funcType = type as any
        const paramType = formatType(funcType.paramType)
        const returnType = formatType(funcType.returnType)
        return `(${paramType}) -> ${returnType}`
      }
      case "GenericType": {
        const genericType = type as any
        const args = genericType.typeArguments.map(formatType).join(", ")
        return `${genericType.name}<${args}>`
      }
      default:
        return type.kind || "unknown"
    }
  }

  const typeStr = formatType(symbolInfo.finalType)
  return `**${symbol}**: ${typeStr}`
}

describe("LSP Type Assertion", () => {
  test("LSPで型アサーション変数の型が正しく取得できる", () => {
    const source = `let x = 42 as String`

    const parser = new Parser(source)
    const parseResult = parser.parse()

    expect(parseResult.errors).toHaveLength(0)
    expect(parseResult.statements).toHaveLength(1)

    const program = new AST.Program(parseResult.statements || [])
    const result = infer(program)

    expect(result.errors).toHaveLength(0)

    // LSPサーバーと同じ方法で変数の型情報を取得
    const symbolInfo = findSymbolWithEnhancedInference(
      program,
      "x",
      result,
      0,
      source
    )

    expect(symbolInfo).toBeDefined()
    expect(symbolInfo.finalType).toBeDefined()
    expect(symbolInfo.finalType.name).toBe("String")

    // フォーマットされた型情報を確認
    const typeInfo = formatInferredTypeInfo("x", symbolInfo)
    console.log("Type info:", typeInfo)
    expect(typeInfo).toBe("**x**: String")
  })

  test("ネストした型アサーションのLSP型情報取得", () => {
    const source = `let z = (42 as Float) as String`

    const parser = new Parser(source)
    const parseResult = parser.parse()

    const program = new AST.Program(parseResult.statements || [])
    const result = infer(program)

    expect(result.errors).toHaveLength(0)

    const symbolInfo = findSymbolWithEnhancedInference(
      program,
      "z",
      result,
      0,
      source
    )

    expect(symbolInfo).toBeDefined()
    expect(symbolInfo.finalType).toBeDefined()
    expect(symbolInfo.finalType.name).toBe("String")

    const typeInfo = formatInferredTypeInfo("z", symbolInfo)
    console.log("Nested type info:", typeInfo)
    expect(typeInfo).toBe("**z**: String")
  })

  test("複数の型アサーション変数", () => {
    const source = `
let x = 42 as String
let y = "hello" as Int
let z = (42 as Float) as String
`

    const parser = new Parser(source)
    const parseResult = parser.parse()

    const program = new AST.Program(parseResult.statements || [])
    const result = infer(program)

    expect(result.errors).toHaveLength(0)

    // x の型確認
    const xInfo = findSymbolWithEnhancedInference(
      program,
      "x",
      result,
      0,
      source
    )
    expect(xInfo?.finalType?.name).toBe("String")

    // y の型確認
    const yInfo = findSymbolWithEnhancedInference(
      program,
      "y",
      result,
      0,
      source
    )
    expect(yInfo?.finalType?.name).toBe("Int")

    // z の型確認
    const zInfo = findSymbolWithEnhancedInference(
      program,
      "z",
      result,
      0,
      source
    )
    expect(zInfo?.finalType?.name).toBe("String")

    console.log("Multiple variables type info:")
    console.log("x:", formatInferredTypeInfo("x", xInfo))
    console.log("y:", formatInferredTypeInfo("y", yInfo))
    console.log("z:", formatInferredTypeInfo("z", zInfo))
  })
})
