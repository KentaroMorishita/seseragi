/**
 * パターンマッチングの制約生成
 */

import * as AST from "../../../ast"
import { TypeConstraint } from "../../constraints"
import {
  addConstraint,
  addError,
  freshTypeVariable,
  type InferenceContext,
} from "../context"
import { generateConstraintsForExpression } from "./dispatcher"

/**
 * パターンに対する制約を生成し、変数を環境にバインド
 */
export function generateConstraintsForPattern(
  ctx: InferenceContext,
  pattern: AST.Pattern,
  expectedType: AST.Type,
  env: Map<string, AST.Type>
): void {
  switch (pattern.kind) {
    case "IdentifierPattern": {
      // 変数パターン: 変数を環境に追加
      const idPattern = pattern as AST.IdentifierPattern
      env.set(idPattern.name, expectedType)
      break
    }

    case "LiteralPattern": {
      // リテラルパターン: 期待する型と一致するかチェック
      const litPattern = pattern as AST.LiteralPattern
      let literalType: AST.Type

      if (litPattern.literalType) {
        switch (litPattern.literalType) {
          case "string":
            literalType = new AST.PrimitiveType(
              "String",
              pattern.line,
              pattern.column
            )
            break
          case "integer":
            literalType = new AST.PrimitiveType(
              "Int",
              pattern.line,
              pattern.column
            )
            break
          case "float":
            literalType = new AST.PrimitiveType(
              "Float",
              pattern.line,
              pattern.column
            )
            break
          case "boolean":
            literalType = new AST.PrimitiveType(
              "Bool",
              pattern.line,
              pattern.column
            )
            break
          default:
            literalType = freshTypeVariable(ctx, pattern.line, pattern.column)
        }
      } else {
        switch (typeof litPattern.value) {
          case "string":
            literalType = new AST.PrimitiveType(
              "String",
              pattern.line,
              pattern.column
            )
            break
          case "number":
            literalType = new AST.PrimitiveType(
              "Int",
              pattern.line,
              pattern.column
            )
            break
          case "boolean":
            literalType = new AST.PrimitiveType(
              "Bool",
              pattern.line,
              pattern.column
            )
            break
          default:
            literalType = freshTypeVariable(ctx, pattern.line, pattern.column)
        }
      }

      addConstraint(
        ctx,
        new TypeConstraint(
          expectedType,
          literalType,
          pattern.line,
          pattern.column,
          `Literal pattern type`
        )
      )
      break
    }

    case "ConstructorPattern": {
      // コンストラクタパターン: 代数的データ型のコンストラクタ
      const ctorPattern = pattern as AST.ConstructorPattern

      const constructorType = env.get(ctorPattern.constructorName)
      if (!constructorType) {
        addError(
          ctx,
          `Unknown constructor: ${ctorPattern.constructorName}`,
          pattern.line,
          pattern.column
        )
        return
      }

      // Function型を辿ってパラメータ型を抽出
      let currentType = constructorType
      const paramTypes: AST.Type[] = []

      while (currentType instanceof AST.FunctionType) {
        paramTypes.push(currentType.paramType)
        currentType = currentType.returnType
      }

      // 最終的な戻り値型（ADT型）
      const adtType = currentType

      addConstraint(
        ctx,
        new TypeConstraint(
          expectedType,
          adtType,
          pattern.line,
          pattern.column,
          `Constructor pattern type`
        )
      )

      if (ctorPattern.patterns.length !== paramTypes.length) {
        addError(
          ctx,
          `Constructor ${ctorPattern.constructorName} expects ${paramTypes.length} arguments, but got ${ctorPattern.patterns.length}`,
          pattern.line,
          pattern.column
        )
        return
      }

      for (let i = 0; i < ctorPattern.patterns.length; i++) {
        const subPattern = ctorPattern.patterns[i]
        const paramType = paramTypes[i]
        if (subPattern && paramType) {
          generateConstraintsForPattern(ctx, subPattern, paramType, env)
        }
      }
      break
    }

    case "WildcardPattern":
      // ワイルドカードパターン: 何にでもマッチし、変数をバインドしない
      break

    case "OrPattern": {
      // orパターン: すべてのサブパターンが同じ型である必要がある
      const orPattern = pattern as AST.OrPattern
      for (const subPattern of orPattern.patterns) {
        generateConstraintsForPattern(ctx, subPattern, expectedType, env)
      }
      break
    }

    case "TuplePattern": {
      // タプルパターン: (x, y, z) = tuple
      const tuplePattern = pattern as AST.TuplePattern

      const expectedElementTypes: AST.Type[] = []
      for (let i = 0; i < tuplePattern.patterns.length; i++) {
        expectedElementTypes.push(
          freshTypeVariable(ctx, pattern.line, pattern.column)
        )
      }

      const tupleType = new AST.TupleType(
        expectedElementTypes,
        pattern.line,
        pattern.column
      )

      addConstraint(
        ctx,
        new TypeConstraint(
          expectedType,
          tupleType,
          pattern.line,
          pattern.column,
          "Tuple pattern structure"
        )
      )

      for (let i = 0; i < tuplePattern.patterns.length; i++) {
        const subPattern = tuplePattern.patterns[i]
        const elemType = expectedElementTypes[i]
        if (subPattern && elemType) {
          generateConstraintsForPattern(ctx, subPattern, elemType, env)
        }
      }
      break
    }

    case "GuardPattern": {
      // ガードパターン: pattern when condition
      const guardPattern = pattern as AST.GuardPattern

      generateConstraintsForPattern(
        ctx,
        guardPattern.pattern,
        expectedType,
        env
      )

      const guardType = generateConstraintsForExpression(
        ctx,
        guardPattern.guard,
        env
      )
      const boolType = new AST.PrimitiveType(
        "Bool",
        pattern.line,
        pattern.column
      )

      addConstraint(
        ctx,
        new TypeConstraint(
          guardType,
          boolType,
          pattern.line,
          pattern.column,
          "Guard condition must be Bool"
        )
      )
      break
    }

    case "ListSugarPattern": {
      // リスト糖衣構文パターン: [|x, y, ...rest|]
      const listSugarPattern = pattern as AST.ListSugarPattern

      const elementTypeVar = freshTypeVariable(
        ctx,
        pattern.line,
        pattern.column
      )
      const listType = new AST.GenericType(
        "List",
        [elementTypeVar],
        pattern.line,
        pattern.column
      )

      addConstraint(
        ctx,
        new TypeConstraint(
          expectedType,
          listType,
          pattern.line,
          pattern.column,
          "List sugar pattern expects List type"
        )
      )

      for (const elemPattern of listSugarPattern.patterns) {
        generateConstraintsForPattern(ctx, elemPattern, elementTypeVar, env)
      }

      if (listSugarPattern.hasRest && listSugarPattern.restPattern) {
        generateConstraintsForPattern(
          ctx,
          listSugarPattern.restPattern,
          expectedType,
          env
        )
      }
      break
    }

    case "ArrayPattern": {
      // 配列パターン: [x, y, ...rest]
      const arrayPattern = pattern as AST.ArrayPattern

      const elementTypeVar = freshTypeVariable(
        ctx,
        pattern.line,
        pattern.column
      )
      const arrayType = new AST.GenericType(
        "Array",
        [elementTypeVar],
        pattern.line,
        pattern.column
      )

      addConstraint(
        ctx,
        new TypeConstraint(
          expectedType,
          arrayType,
          pattern.line,
          pattern.column,
          "Array pattern expects Array type"
        )
      )

      for (const elemPattern of arrayPattern.patterns) {
        generateConstraintsForPattern(ctx, elemPattern, elementTypeVar, env)
      }

      if (arrayPattern.hasRest && arrayPattern.restPattern) {
        generateConstraintsForPattern(
          ctx,
          arrayPattern.restPattern,
          expectedType,
          env
        )
      }
      break
    }

    default:
      addError(
        ctx,
        `Unsupported pattern type: ${pattern.kind}`,
        pattern.line,
        pattern.column
      )
  }
}
