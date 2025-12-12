/**
 * Import Declaration Generator Tests
 */

import { describe, expect, test } from "bun:test"
import { ImportDeclaration, ImportItem } from "../src/ast"
import { createContext } from "../src/codegen/context"
import { generateImportDeclaration } from "../src/codegen/generators/statements/import-declaration"

describe("Import Declaration Generator", () => {
  test("モジュール解決が利用できない場合", () => {
    const ctx = createContext({})
    const importDecl = new ImportDeclaration(
      "./math",
      [new ImportItem("add", undefined, 1, 1)],
      1,
      1
    )

    const result = generateImportDeclaration(ctx, importDecl)
    expect(result).toBe("// Import resolution not available: ./math")
  })

  test("モジュール解決が失敗した場合", () => {
    const ctx = createContext({
      typeInferenceResult: {
        moduleResolver: {
          resolve: (): null => null,
        },
        currentFilePath: "/test/file.ssrg",
      } as any,
    })

    const importDecl = new ImportDeclaration(
      "./nonexistent",
      [new ImportItem("foo", undefined, 1, 1)],
      1,
      1
    )

    const result = generateImportDeclaration(ctx, importDecl)
    expect(result).toBe("// Failed to resolve module: ./nonexistent")
  })

  test("インポート成功時にコメントヘッダーを生成", () => {
    const ctx = createContext({
      typeInferenceResult: {
        moduleResolver: {
          resolve: () => ({
            exports: {
              functions: new Map(),
              types: new Map(),
              impls: new Map(),
            },
          }),
        },
        currentFilePath: "/test/file.ssrg",
      } as any,
    })

    const importDecl = new ImportDeclaration(
      "./math",
      [new ImportItem("add", undefined, 1, 1)],
      1,
      1
    )

    const result = generateImportDeclaration(ctx, importDecl)
    expect(result).toContain("// Inlined from module: ./math")
  })

  test("存在しないエクスポートの警告", () => {
    const ctx = createContext({
      typeInferenceResult: {
        moduleResolver: {
          resolve: () => ({
            exports: {
              functions: new Map(),
              types: new Map(),
              impls: new Map(),
            },
          }),
        },
        currentFilePath: "/test/file.ssrg",
      } as any,
    })

    const importDecl = new ImportDeclaration(
      "./math",
      [new ImportItem("nonexistent", undefined, 1, 1)],
      1,
      1
    )

    const result = generateImportDeclaration(ctx, importDecl)
    expect(result).toContain(
      "// Warning: Could not find export 'nonexistent' in module ./math"
    )
  })

  test("複数のインポート項目", () => {
    const ctx = createContext({
      typeInferenceResult: {
        moduleResolver: {
          resolve: () => ({
            exports: {
              functions: new Map(),
              types: new Map(),
              impls: new Map(),
            },
          }),
        },
        currentFilePath: "/test/file.ssrg",
      } as any,
    })

    const importDecl = new ImportDeclaration(
      "./utils",
      [
        new ImportItem("helper1", undefined, 1, 1),
        new ImportItem("helper2", undefined, 1, 1),
      ],
      1,
      1
    )

    const result = generateImportDeclaration(ctx, importDecl)
    expect(result).toContain("// Inlined from module: ./utils")
    expect(result).toContain(
      "// Warning: Could not find export 'helper1' in module ./utils"
    )
    expect(result).toContain(
      "// Warning: Could not find export 'helper2' in module ./utils"
    )
  })
})
