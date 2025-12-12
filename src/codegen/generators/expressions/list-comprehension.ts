/**
 * リスト内包表記の生成
 */

import type { ListComprehension, ListComprehensionSugar } from "../../../ast"
import type { CodeGenContext } from "../../context"
import { generateExpression } from "../dispatcher"

/**
 * リスト内包表記をTypeScriptコードに変換
 * [x * 2 | x <- range, x % 2 == 0] ->
 * range.filter(x => x % 2 == 0).map(x => x * 2)
 */
export function generateListComprehension(
  ctx: CodeGenContext,
  comp: ListComprehension
): string {
  let result = ""

  // 最初のジェネレータから開始
  if (comp.generators.length > 0) {
    const firstGenerator = comp.generators[0]!
    result = generateExpression(ctx, firstGenerator.iterable)

    // 追加のジェネレータがある場合はflatMapを使用
    for (let i = 1; i < comp.generators.length; i++) {
      const generator = comp.generators[i]!
      const iterable = generateExpression(ctx, generator.iterable)
      result = `${result}.flatMap(${firstGenerator.variable} => ${iterable}.map(${generator.variable} => [${firstGenerator.variable}, ${generator.variable}]))`
    }

    // フィルタを適用
    for (const filter of comp.filters) {
      const filterExpr = generateExpression(ctx, filter)
      // ジェネレータ変数を適切に置換
      if (comp.generators.length === 1) {
        result = `${result}.filter(${comp.generators[0]!.variable} => ${filterExpr})`
      } else {
        // 複数ジェネレータの場合は複雑になるので簡略化
        result = `${result}.filter(tuple => {
          const [${comp.generators.map((g) => g.variable).join(", ")}] = tuple;
          return ${filterExpr};
        })`
      }
    }

    // 最終的な式をマップ
    const expression = generateExpression(ctx, comp.expression)
    if (comp.generators.length === 1) {
      result = `${result}.map(${comp.generators[0]!.variable} => ${expression})`
    } else {
      result = `${result}.map(tuple => {
        const [${comp.generators.map((g) => g.variable).join(", ")}] = tuple;
        return ${expression};
      })`
    }
  }

  return result || "[]"
}

/**
 * リスト内包表記（Sugar版）をTypeScriptコードに変換
 * Seseragiリストを返す
 */
export function generateListComprehensionSugar(
  ctx: CodeGenContext,
  comp: ListComprehensionSugar
): string {
  // まず通常の配列内包表記を生成
  let arrayResult = ""

  // 最初のジェネレータから開始
  if (comp.generators.length > 0) {
    const firstGenerator = comp.generators[0]!
    arrayResult = generateExpression(ctx, firstGenerator.iterable)

    // 追加のジェネレータがある場合はflatMapを使用
    for (let i = 1; i < comp.generators.length; i++) {
      const generator = comp.generators[i]!
      const iterable = generateExpression(ctx, generator.iterable)
      arrayResult = `${arrayResult}.flatMap(${firstGenerator.variable} => ${iterable}.map(${generator.variable} => [${firstGenerator.variable}, ${generator.variable}]))`
    }

    // フィルタを適用
    for (const filter of comp.filters) {
      const filterExpr = generateExpression(ctx, filter)
      // ジェネレータ変数を適切に置換
      if (comp.generators.length === 1) {
        arrayResult = `${arrayResult}.filter(${comp.generators[0]!.variable} => ${filterExpr})`
      } else {
        // 複数ジェネレータの場合は複雑になるので簡略化
        arrayResult = `${arrayResult}.filter(tuple => {
          const [${comp.generators.map((g) => g.variable).join(", ")}] = tuple;
          return ${filterExpr};
        })`
      }
    }

    // 最終的な式をマップ
    const expression = generateExpression(ctx, comp.expression)
    if (comp.generators.length === 1) {
      arrayResult = `${arrayResult}.map(${comp.generators[0]!.variable} => ${expression})`
    } else {
      arrayResult = `${arrayResult}.map(tuple => {
        const [${comp.generators.map((g) => g.variable).join(", ")}] = tuple;
        return ${expression};
      })`
    }
  }

  if (!arrayResult) {
    return "Empty"
  }

  // 配列をSeseragiリストに変換
  return `arrayToList(${arrayResult})`
}
