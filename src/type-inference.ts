/**
 * 型推論システム (Type Inference System) for Seseragi Language
 *
 * Hindley-Milner型推論アルゴリズムを実装
 */

import * as AST from "./ast"
import {
  ApplicativeApplyConstraint,
  ArrayAccessConstraint,
  FunctorMapConstraint,
  SubtypeConstraint,
  TypeConstraint,
} from "./inference/constraints"
import { createInitialEnvironment as createInitialEnvironmentUtil } from "./inference/environment"
import { TypeInferenceError } from "./inference/errors"
import { TypeSubstitution } from "./inference/substitution"
import {
  createFlattenedUnionType as createFlattenedUnionTypeUtil,
  isRecordSubset as isRecordSubsetUtil,
  isRecordSubtype as isRecordSubtypeUtil,
  mergeRecordTypes as mergeRecordTypesUtil,
  typesEqual as typesEqualUtil,
} from "./inference/type-comparison"
import {
  formatType as formatTypeUtil,
  typeToCanonicalString as typeToCanonicalStringUtil,
  typeToString as typeToStringUtil,
} from "./inference/type-formatter"
import {
  collectPolymorphicTypeVariables as collectPolymorphicTypeVariablesUtil,
  getTypeName as getTypeNameUtil,
  isEitherType as isEitherTypeUtil,
  isMaybeType as isMaybeTypeUtil,
  isPromiseType as isPromiseTypeUtil,
  occursCheck as occursCheckUtil,
} from "./inference/type-inspection"
import {
  collectTypeVariables as collectTypeVariablesUtil,
  getFreeTypeVariables as getFreeTypeVariablesUtil,
  substituteTypeVariables as substituteTypeVariablesUtil,
} from "./inference/type-substitution-utils"
import {
  PolymorphicTypeVariable,
  TypeVariable,
} from "./inference/type-variables"

// re-export for external consumers
export { TypeVariable, PolymorphicTypeVariable }
export {
  SubtypeConstraint,
  TypeConstraint,
  ArrayAccessConstraint,
  FunctorMapConstraint,
  ApplicativeApplyConstraint,
}
export { TypeSubstitution }
export { TypeInferenceError }
export { formatTypeUtil as formatType, typeToStringUtil as typeToString }

// ブラウザ環境でもエラーが出ないように条件分岐
let ModuleResolver: any
if (
  typeof globalThis !== "undefined" &&
  (globalThis as any).window === undefined
) {
  // Node.js環境
  ModuleResolver = require("./module-resolver").ModuleResolver
} else {
  // ブラウザ環境：ダミークラス
  ModuleResolver = class {
    resolve(): null {
      return null
    }
    clearCache(): void {}
  }
}

// 型推論結果インターフェース
export interface InferenceResult {
  errors: TypeInferenceError[]
  inferredTypes?: Map<string, AST.Type>
  typeEnvironment?: Map<string, AST.Type>
}

// TypeInferenceSystemのinferメソッドの戻り値型
export interface TypeInferenceSystemResult {
  substitution: TypeSubstitution
  errors: TypeInferenceError[]
  nodeTypeMap: Map<AST.ASTNode, AST.Type>
  moduleResolver?: typeof ModuleResolver
  currentFilePath?: string
  environment: Map<string, AST.Type>
}

// 型推論システムのメインクラス
export class TypeInferenceSystem {
  private nextVarId = 1000 // Start from 1000 to avoid conflicts with parser-generated type variables
  private constraints: (
    | TypeConstraint
    | ArrayAccessConstraint
    | FunctorMapConstraint
    | ApplicativeApplyConstraint
  )[] = []
  private subtypeConstraints: SubtypeConstraint[] = []
  private errors: TypeInferenceError[] = []
  private nodeTypeMap: Map<AST.ASTNode, AST.Type> = new Map() // Track types for AST nodes
  private methodEnvironment: Map<string, AST.MethodDeclaration> = new Map() // Track methods by type.method
  private currentProgram: AST.Program | null = null // 現在処理中のプログラム
  private typeAliases: Map<string, AST.Type> = new Map() // 型エイリアス情報を保持
  private moduleResolver: any = new ModuleResolver()
  private currentFilePath: string = ""

  // 新しい型変数を生成
  freshTypeVariable(line: number, column: number): TypeVariable {
    return new TypeVariable(this.nextVarId++, line, column)
  }

  // エラーを追加
  addError(
    message: string,
    line: number,
    column: number,
    context?: string
  ): void {
    this.errors.push(new TypeInferenceError(message, line, column, context))
  }

  // 型エイリアス情報を設定
  setTypeAliases(typeAliases: Map<string, AST.Type>): void {
    this.typeAliases = typeAliases
  }

  // 再帰的に型エイリアスを解決
  private resolveTypeAliasRecursively(
    typeName: string,
    visited: Set<string> = new Set()
  ): AST.Type | null {
    if (visited.has(typeName)) {
      // 循環参照を防ぐ
      return null
    }

    const aliasData = this.typeAliases.get(typeName)
    if (!aliasData) {
      return null
    }

    // TypeAliasDeclarationの場合は、その中のaliasedTypeを取得
    let aliasedType: AST.Type
    if (aliasData.kind === "TypeAliasDeclaration") {
      aliasedType = (aliasData as AST.TypeAliasDeclaration).aliasedType
    } else {
      aliasedType = aliasData
    }

    // RecordTypeなら直接返す
    if (aliasedType.kind === "RecordType") {
      return aliasedType
    }

    // 別のPrimitiveType（型エイリアス）なら再帰的に解決
    if (aliasedType.kind === "PrimitiveType") {
      visited.add(typeName)
      const nextTypeName = (aliasedType as AST.PrimitiveType).name
      return this.resolveTypeAliasRecursively(nextTypeName, visited)
    }

    return aliasedType
  }

  // IntersectionTypeからRecordTypeを抽出・統合
  private extractRecordFromIntersection(
    intersectionType: AST.IntersectionType
  ): AST.RecordType | null {
    const allFields: AST.RecordField[] = []

    for (const type of intersectionType.types) {
      const recordType = this.extractRecordTypeFromAnyType(type)
      if (recordType) {
        allFields.push(...recordType.fields)
      }
    }

    if (allFields.length === 0) {
      return null
    }

    // 統合されたRecordTypeを作成
    return new AST.RecordType(
      allFields,
      intersectionType.line,
      intersectionType.column
    )
  }

  // 任意の型からRecordTypeを抽出する汎用メソッド
  private extractRecordTypeFromAnyType(type: AST.Type): AST.RecordType | null {
    switch (type.kind) {
      case "RecordType":
        return type as AST.RecordType

      case "PrimitiveType": {
        // 型エイリアスを解決
        const resolvedType = this.resolveTypeAliasRecursively(
          (type as AST.PrimitiveType).name
        )
        return resolvedType
          ? this.extractRecordTypeFromAnyType(resolvedType)
          : null
      }

      case "IntersectionType":
        // ネストしたIntersectionTypeも処理
        return this.extractRecordFromIntersection(type as AST.IntersectionType)

      default:
        // これらの型からはRecordTypeは抽出できない
        return null
    }
  }

  // 型の一般化（generalization）- フリー型変数を多相型変数に変換
  generalize(type: AST.Type, env: Map<string, AST.Type>): AST.Type {
    const freeVars = getFreeTypeVariablesUtil(type, env)
    if (freeVars.size === 0) {
      return type
    }

    const substitutionMap = new Map<string, AST.Type>()
    let polyVarIndex = 0

    // フリー型変数を多相型変数に置換
    for (const varName of freeVars) {
      const polyVarName = String.fromCharCode(97 + polyVarIndex) // 'a', 'b', 'c', ...
      substitutionMap.set(
        varName,
        new PolymorphicTypeVariable(polyVarName, type.line, type.column)
      )
      polyVarIndex++
    }

    return substituteTypeVariablesUtil(type, substitutionMap)
  }

  // ジェネリック型エイリアス専用の型変数置換
  private substituteTypeVariablesInGenericAlias(
    type: AST.Type,
    typeParameters: AST.TypeParameter[],
    typeArguments: AST.Type[]
  ): AST.Type {
    // 型パラメータの名前と型引数の位置を使って置換マップを作成
    const substitutionMap = new Map<string, AST.Type>()
    for (
      let i = 0;
      i < typeParameters.length && i < typeArguments.length;
      i++
    ) {
      substitutionMap.set(typeParameters[i].name, typeArguments[i])
    }

    // 型エイリアスの定義内で見つかった型変数を、型パラメータの位置に基づいて置換
    const typeVariablesToReplace = new Set<TypeVariable>()
    collectTypeVariablesUtil(type, typeVariablesToReplace)

    // 型変数を位置ベースで置換するマップを作成
    const typeVariableArray = Array.from(typeVariablesToReplace).sort(
      (a, b) => a.id - b.id
    )
    const positionMap = new Map<number, AST.Type>()
    for (
      let i = 0;
      i < typeVariableArray.length && i < typeArguments.length;
      i++
    ) {
      positionMap.set(typeVariableArray[i].id, typeArguments[i])
    }

    const substitute = (t: AST.Type): AST.Type => {
      switch (t.kind) {
        case "TypeVariable": {
          const tv = t as TypeVariable
          // 型変数のIDを使って対応する型引数を取得
          const substituted = positionMap.get(tv.id)
          if (substituted) {
            return substituted
          }
          return t
        }
        case "TupleType": {
          const tt = t as AST.TupleType
          return new AST.TupleType(
            tt.elementTypes.map(substitute),
            tt.line,
            tt.column
          )
        }
        case "RecordType": {
          const rt = t as AST.RecordType
          return new AST.RecordType(
            rt.fields.map(
              (f) =>
                new AST.RecordField(
                  f.name,
                  substitute(f.type),
                  f.line,
                  f.column
                )
            ),
            rt.line,
            rt.column
          )
        }
        case "FunctionType": {
          const ft = t as AST.FunctionType
          return new AST.FunctionType(
            substitute(ft.paramType),
            substitute(ft.returnType),
            ft.line,
            ft.column
          )
        }
        case "GenericType": {
          const gt = t as AST.GenericType
          return new AST.GenericType(
            gt.name,
            gt.typeArguments.map(substitute),
            gt.line,
            gt.column
          )
        }
        case "PrimitiveType": {
          const pt = t as AST.PrimitiveType
          // 型パラメータ名が一致する場合は置換
          const substituted = substitutionMap.get(pt.name)
          if (substituted) {
            return substituted
          }
          return t
        }
        default:
          return t
      }
    }

    return substitute(type)
  }

  // 多相型を具体化（インスタンス化）
  instantiatePolymorphicType(
    type: AST.Type,
    line: number,
    column: number
  ): AST.Type {
    const substitutionMap = new Map<string, AST.Type>()

    const substitute = (t: AST.Type): AST.Type => {
      switch (t.kind) {
        case "PolymorphicTypeVariable": {
          const polyVar = t as PolymorphicTypeVariable
          if (!substitutionMap.has(polyVar.name)) {
            substitutionMap.set(
              polyVar.name,
              this.freshTypeVariable(line, column)
            )
          }
          return substitutionMap.get(polyVar.name)!
        }

        case "FunctionType": {
          const ft = t as AST.FunctionType
          return new AST.FunctionType(
            substitute(ft.paramType),
            substitute(ft.returnType),
            ft.line,
            ft.column
          )
        }

        case "GenericType": {
          const gt = t as AST.GenericType
          return new AST.GenericType(
            gt.name,
            gt.typeArguments.map((arg) => substitute(arg)),
            gt.line,
            gt.column
          )
        }

        case "RecordType": {
          const rt = t as AST.RecordType
          return new AST.RecordType(
            rt.fields.map(
              (field) =>
                new AST.RecordField(
                  field.name,
                  substitute(field.type),
                  field.line,
                  field.column
                )
            ),
            rt.line,
            rt.column
          )
        }

        case "StructType": {
          const st = t as AST.StructType
          return new AST.StructType(
            st.name,
            st.fields.map(
              (field) =>
                new AST.RecordField(
                  field.name,
                  substitute(field.type),
                  field.line,
                  field.column
                )
            ),
            st.line,
            st.column
          )
        }

        default:
          return t
      }
    }

    return substitute(type)
  }

  // 明示的型引数で多相型を具体化
  instantiateWithExplicitTypeArguments(
    type: AST.Type,
    typeArguments: AST.Type[],
    line: number,
    column: number
  ): AST.Type {
    // 多相型変数を収集
    const polymorphicVars = collectPolymorphicTypeVariablesUtil(type)

    // 型引数の数が一致しない場合はエラー
    if (polymorphicVars.length !== typeArguments.length) {
      throw new Error(
        `Type argument count mismatch at ${line}:${column}. Expected ${polymorphicVars.length} but got ${typeArguments.length}`
      )
    }

    // 多相型変数を明示的型引数で置換するマップを作成
    const substitutionMap = new Map<string, AST.Type>()
    for (let i = 0; i < polymorphicVars.length; i++) {
      substitutionMap.set(polymorphicVars[i], typeArguments[i])
    }

    return substituteTypeVariablesUtil(type, substitutionMap)
  }

  // 制約を追加
  addConstraint(constraint: TypeConstraint): void {
    this.constraints.push(constraint)
  }

  addSubtypeConstraint(constraint: SubtypeConstraint): void {
    this.subtypeConstraints.push(constraint)
  }

  // 型推論のメインエントリーポイント
  infer(program: AST.Program, filePath?: string): TypeInferenceSystemResult {
    this.constraints = []
    this.subtypeConstraints = []
    this.currentEnvironment.clear() // 環境をクリア
    this.typeAliases.clear() // ジェネリック型エイリアス情報をクリア
    this.typeAliasParameterMappings.clear() // 型パラメータマッピングをクリア
    this.errors = []
    this.nextVarId = 1000 // Reset to 1000 to avoid conflicts with parser-generated type variables
    this.nodeTypeMap.clear()
    this.currentProgram = program // プログラム情報を保存
    this.currentFilePath = filePath || ""

    // 型環境の初期化
    const env = createInitialEnvironmentUtil()

    // 制約生成
    this.generateConstraints(program, env)

    // 制約解決（単一化）
    const substitution = this.solveConstraints()

    // Apply substitution to all tracked node types
    const resolvedNodeTypeMap = new Map<AST.ASTNode, AST.Type>()
    for (const [node, type] of this.nodeTypeMap) {
      resolvedNodeTypeMap.set(node, substitution.apply(type))
    }

    return {
      substitution,
      errors: this.errors,
      nodeTypeMap: resolvedNodeTypeMap,
      moduleResolver: this.moduleResolver,
      currentFilePath: this.currentFilePath,
      environment: new Map(this.currentEnvironment),
    }
  }

  // 制約生成
  private generateConstraints(
    program: AST.Program,
    env: Map<string, AST.Type>
  ): void {
    // 現在の環境を設定（型エイリアス解決用）
    this.currentEnvironment = env

    // Two-pass approach to handle forward references:
    // Pass 1: Process imports first, then all function declarations, type declarations, and struct declarations
    // This allows variables to reference functions and types defined later in the file
    for (const statement of program.statements) {
      if (
        statement.kind === "ImportDeclaration" ||
        statement.kind === "FunctionDeclaration" ||
        statement.kind === "TypeDeclaration" ||
        statement.kind === "TypeAliasDeclaration" ||
        statement.kind === "StructDeclaration"
      ) {
        this.generateConstraintsForStatement(statement, env)
      }
    }

    // Pass 2: Process impl blocks, variable declarations and expression statements in original order
    // At this point all functions and types are available in the environment
    for (const statement of program.statements) {
      if (statement.kind === "ImplBlock") {
        this.generateConstraintsForStatement(statement, env)
      }
    }

    // Pass 3: Process variable declarations and expression statements
    for (const statement of program.statements) {
      if (
        statement.kind === "VariableDeclaration" ||
        statement.kind === "ExpressionStatement" ||
        statement.kind === "TupleDestructuring" ||
        statement.kind === "RecordDestructuring" ||
        statement.kind === "StructDestructuring"
      ) {
        this.generateConstraintsForStatement(statement, env)
      }
    }
  }

  private generateConstraintsForStatement(
    statement: AST.Statement,
    env: Map<string, AST.Type>
  ): void {
    switch (statement.kind) {
      case "ImportDeclaration":
        this.generateConstraintsForImportDeclaration(
          statement as AST.ImportDeclaration,
          env
        )
        break
      case "FunctionDeclaration":
        this.generateConstraintsForFunctionDeclaration(
          statement as AST.FunctionDeclaration,
          env
        )
        break
      case "VariableDeclaration":
        this.generateConstraintsForVariableDeclaration(
          statement as AST.VariableDeclaration,
          env
        )
        break
      case "ExpressionStatement":
        this.generateConstraintsForExpression(
          (statement as AST.ExpressionStatement).expression,
          env
        )
        break
      case "TypeDeclaration":
        this.generateConstraintsForTypeDeclaration(
          statement as AST.TypeDeclaration,
          env
        )
        break
      case "TypeAliasDeclaration":
        this.generateConstraintsForTypeAliasDeclaration(
          statement as AST.TypeAliasDeclaration,
          env
        )
        break
      case "TupleDestructuring":
        this.generateConstraintsForTupleDestructuring(
          statement as AST.TupleDestructuring,
          env
        )
        break
      case "RecordDestructuring":
        this.generateConstraintsForRecordDestructuring(
          statement as AST.RecordDestructuring,
          env
        )
        break
      case "StructDestructuring":
        this.generateConstraintsForStructDestructuring(
          statement as AST.StructDestructuring,
          env
        )
        break
      case "StructDeclaration":
        this.generateConstraintsForStructDeclaration(
          statement as AST.StructDeclaration,
          env
        )
        break
      case "ImplBlock":
        this.generateConstraintsForImplBlock(statement as AST.ImplBlock, env)
        break
      default:
        // 他の文の種類は後で実装
        break
    }
  }

  private generateConstraintsForFunctionDeclaration(
    func: AST.FunctionDeclaration,
    env: Map<string, AST.Type>
  ): void {
    // 型パラメータを多相型変数として扱うマップを作成
    const typeParameterMap = new Map<string, AST.Type>()
    if (func.typeParameters) {
      for (const typeParam of func.typeParameters) {
        typeParameterMap.set(
          typeParam.name,
          new PolymorphicTypeVariable(
            typeParam.name,
            typeParam.line,
            typeParam.column
          )
        )
      }
    }

    // 型の解決において、型パラメータを多相型変数で置換
    const resolveTypeWithTypeParameters = (
      type: AST.Type | undefined
    ): AST.Type => {
      if (!type) {
        return this.freshTypeVariable(func.line, func.column)
      }
      return substituteTypeVariablesUtil(type, typeParameterMap)
    }

    // 戻り値の型が指定されていない場合は型変数を作成
    let returnType = resolveTypeWithTypeParameters(func.returnType)

    // 型エイリアスの解決
    if (returnType) {
      returnType = this.resolveTypeAlias(returnType)
    }

    // パラメータの型を事前に決定
    const paramTypes: AST.Type[] = []
    for (const param of func.parameters) {
      const paramType = resolveTypeWithTypeParameters(param.type)
      paramTypes.push(paramType)
    }

    // 関数シグネチャを構築
    let funcType: AST.Type = returnType

    // パラメータから関数シグネチャを構築（カリー化）
    if (func.parameters.length === 0) {
      // 引数なしの関数は Unit -> ReturnType
      const unitType = new AST.PrimitiveType("Unit", func.line, func.column)
      funcType = new AST.FunctionType(
        unitType,
        funcType,
        func.line,
        func.column
      )
    } else {
      // 引数ありの関数は通常のカリー化（後ろから前に構築）
      for (let i = paramTypes.length - 1; i >= 0; i--) {
        funcType = new AST.FunctionType(
          paramTypes[i],
          funcType,
          func.line,
          func.column
        )
      }
    }

    // 関数を環境に追加
    const generalizedType = this.generalize(funcType, env)
    env.set(func.name, generalizedType)

    // 関数本体の型推論用の環境を作成
    const bodyEnv = new Map(env)

    // パラメータの型を環境に追加（型エイリアス解決後）
    for (let i = 0; i < func.parameters.length; i++) {
      const param = func.parameters[i]
      const paramType = paramTypes[i]

      // パラメータ型も型エイリアス解決を行う
      const resolvedParamType = this.resolveTypeAlias(paramType)
      bodyEnv.set(param.name, resolvedParamType)
    }

    // 関数本体の型を推論
    const bodyType = this.generateConstraintsForExpression(
      func.body,
      bodyEnv,
      returnType
    )

    // 関数本体の型と戻り値型が一致することを制約として追加
    this.addConstraint(
      new TypeConstraint(
        bodyType,
        returnType,
        func.body.line,
        func.body.column,
        `Function ${func.name} body type`
      )
    )
  }

  private generateConstraintsForImportDeclaration(
    importDecl: AST.ImportDeclaration,
    env: Map<string, AST.Type>
  ): void {
    // キャッシュをクリアして最新の内容を読み込む
    this.moduleResolver.clearCache()

    // モジュールを解決
    const resolvedModule = this.moduleResolver.resolve(
      importDecl.module,
      this.currentFilePath
    )

    if (!resolvedModule) {
      this.addError(
        `Cannot resolve module '${importDecl.module}'`,
        importDecl.line,
        importDecl.column
      )
      return
    }

    // インポートした項目を環境に追加（依存順序で処理：型 -> 関数 -> 構造体）

    // Phase 1: ADT types first (no dependencies)
    for (const item of importDecl.items) {
      const exportedType = resolvedModule.exports.types.get(item.name)
      if (exportedType && exportedType.kind === "TypeDeclaration") {
        const importName = item.alias || item.name
        const typeDecl = exportedType as AST.TypeDeclaration

        // Union型を作成してADT型として環境に追加
        const unionTypes = typeDecl.fields.map(
          (field) =>
            new AST.PrimitiveType(
              field.name,
              field.line || 0,
              field.column || 0
            )
        )
        const unionType = new AST.UnionType(
          unionTypes,
          typeDecl.line,
          typeDecl.column
        )
        env.set(importName, unionType)

        // 各コンストラクタも環境に追加
        for (const field of typeDecl.fields) {
          const constructorType = new AST.PrimitiveType(
            field.name,
            field.line || 0,
            field.column || 0
          )
          env.set(field.name, constructorType)
        }
      }
    }

    // Phase 2: Other types and functions
    for (const item of importDecl.items) {
      const exportedFunction = resolvedModule.exports.functions.get(item.name)
      const exportedType = resolvedModule.exports.types.get(item.name)

      if (exportedFunction) {
        // 関数をインポート
        const funcType =
          this.createFunctionTypeFromDeclaration(exportedFunction)
        const importName = item.alias || item.name
        env.set(importName, funcType)
      } else if (exportedType) {
        // 型をインポート
        const importName = item.alias || item.name

        // 型エイリアスの場合は、その定義する型を取得する
        if (exportedType && exportedType.kind === "TypeAliasDeclaration") {
          const aliasDecl = exportedType as AST.TypeAliasDeclaration
          env.set(importName, aliasDecl.aliasedType)
        } else if (exportedType.kind === "TypeDeclaration") {
          // ADTは既にPhase 1で処理済み
        } else if (exportedType.kind === "StructDeclaration") {
          // StructDeclarationの場合は、StructTypeに変換してインポート
          const structDecl = exportedType as AST.StructDeclaration
          const structType = new AST.StructType(
            structDecl.name,
            structDecl.fields,
            structDecl.line,
            structDecl.column
          )
          env.set(importName, structType)

          // 対応するimpl定義も取得してメソッドを登録
          const implBlock = resolvedModule.exports.impls.get(structDecl.name)
          if (implBlock) {
            // ローカルのImplBlockと同じように制約を生成（制約生成は後で実行）
            // TODO: 制約生成をインポート完了後に実行するように変更
            for (const method of implBlock.methods) {
              this.generateConstraintsForMethodDeclaration(
                method,
                env,
                structType
              )
            }
            for (const operator of implBlock.operators) {
              this.generateConstraintsForOperatorDeclaration(
                operator,
                env,
                structType
              )
            }
          }
        } else {
          // その他の型はそのまま
          env.set(importName, exportedType as AST.Type)
        }
      } else {
        // エクスポートされていないアイテム
        this.addError(
          `Module '${importDecl.module}' does not export '${item.name}'`,
          importDecl.line,
          importDecl.column
        )
      }
    }
  }

  private createFunctionTypeFromDeclaration(
    funcDecl: AST.FunctionDeclaration
  ): AST.Type {
    // 関数の型を作成（パラメータ → 戻り値）
    let resultType = funcDecl.returnType

    // カリー化された関数型を構築（右結合）
    for (let i = funcDecl.parameters.length - 1; i >= 0; i--) {
      const param = funcDecl.parameters[i]
      resultType = new AST.FunctionType(
        param.type,
        resultType,
        funcDecl.line,
        funcDecl.column
      )
    }

    return resultType
  }

  private generateConstraintsForVariableDeclaration(
    varDecl: AST.VariableDeclaration,
    env: Map<string, AST.Type>
  ): AST.Type {
    // 型注釈がある場合は期待される型として渡す
    let expectedType: AST.Type | undefined
    if (varDecl.type) {
      expectedType = varDecl.type
      if (varDecl.type.kind === "PrimitiveType") {
        const aliasedType = env.get(varDecl.type.name)
        if (aliasedType) {
          expectedType = aliasedType
        }
      }
    }

    // 制約生成前の状態を記録
    const constraintsBeforeInit = this.constraints.length

    // 初期化式の型を推論
    const initType = this.generateConstraintsForExpression(
      varDecl.initializer,
      env,
      expectedType
    )

    let finalType: AST.Type
    if (varDecl.type) {
      // 明示的な型注釈がある場合（expectedTypeは既に解決済み）
      const resolvedType = expectedType!

      // 制約を追加（解決された型で）
      this.addConstraint(
        new TypeConstraint(
          initType,
          resolvedType,
          varDecl.line,
          varDecl.column,
          `Variable ${varDecl.name} type annotation`
        )
      )
      env.set(varDecl.name, resolvedType)
      finalType = resolvedType
    } else {
      // 型注釈がない場合の処理
      // 初期化式がラムダ式の場合のみ一般化、それ以外は推論された型をそのまま使用
      if (varDecl.initializer.kind === "LambdaExpression") {
        // ラムダ式は多相性を保つため一般化
        const generalizedType = this.generalize(initType, env)
        env.set(varDecl.name, generalizedType)
        finalType = generalizedType
      } else {
        // 値型の場合：段階的制約解決を試行
        const newConstraints = this.constraints.slice(constraintsBeforeInit)

        // 新しい制約のみを解決してinitTypeを具体化を試行
        const partialSubstitution = this.solveConstraintsPartial(newConstraints)
        const resolvedInitType = partialSubstitution.apply(initType)

        // 解決された型を環境に設定
        env.set(varDecl.name, resolvedInitType)
        finalType = resolvedInitType
      }
    }

    // Track the type for this variable declaration
    this.nodeTypeMap.set(varDecl, finalType)

    // Also track the initializer expression type
    this.nodeTypeMap.set(varDecl.initializer, initType)

    return finalType
  }

  // 構造体の演算子定義を検索するヘルパー関数
  private findOperatorDefinition(
    structType: AST.Type,
    operator: string,
    _env: Map<string, AST.Type>
  ): AST.OperatorDeclaration | null {
    if (!this.isValidStructType(structType)) {
      return null
    }

    const structTypeName = (structType as AST.StructType).name
    return this.searchOperatorInImplBlocks(structTypeName, operator)
  }

  private isValidStructType(structType: AST.Type): boolean {
    return structType.kind === "StructType" && !!this.currentProgram
  }

  private searchOperatorInImplBlocks(
    structTypeName: string,
    operator: string
  ): AST.OperatorDeclaration | null {
    for (const stmt of this.currentProgram!.statements) {
      if (stmt.kind === "ImplBlock") {
        const result = this.searchOperatorInImplBlock(
          stmt as AST.ImplBlock,
          structTypeName,
          operator
        )
        if (result) {
          return result
        }
      }
    }
    return null
  }

  private searchOperatorInImplBlock(
    implBlock: AST.ImplBlock,
    structTypeName: string,
    operator: string
  ): AST.OperatorDeclaration | null {
    if (implBlock.typeName !== structTypeName) {
      return null
    }

    for (const op of implBlock.operators) {
      if (op.operator === operator) {
        return op
      }
    }
    return null
  }

  private generateConstraintsForTupleDestructuring(
    tupleDestr: AST.TupleDestructuring,
    env: Map<string, AST.Type>
  ): void {
    // 初期化式の型を推論
    const initType = this.generateConstraintsForExpression(
      tupleDestr.initializer,
      env
    )

    // タプルパターンを処理して変数を環境に追加
    this.generateConstraintsForPattern(tupleDestr.pattern, initType, env)

    // ノード型マップに情報を記録
    this.nodeTypeMap.set(tupleDestr, initType)
    this.nodeTypeMap.set(tupleDestr.initializer, initType)
  }

  private generateConstraintsForTypeDeclaration(
    typeDecl: AST.TypeDeclaration,
    env: Map<string, AST.Type>
  ): void {
    // ビルトイン型との名前衝突をチェック
    const builtinTypes = ["Maybe", "Either", "List"]
    if (builtinTypes.includes(typeDecl.name)) {
      const error = new TypeInferenceError(
        `Type '${typeDecl.name}' conflicts with builtin type. Use a different name.`,
        typeDecl.line,
        typeDecl.column
      )
      this.errors.push(error)
      return
    }

    // ADT型を環境に追加
    const adtType = new AST.PrimitiveType(
      typeDecl.name,
      typeDecl.line,
      typeDecl.column
    )

    // ADT型自体を環境に登録
    env.set(typeDecl.name, adtType)

    // 各バリアント（コンストラクタ）を環境に追加
    for (const field of typeDecl.fields) {
      const constructorType = this.createConstructorType(field, adtType)
      env.set(field.name, constructorType)
    }
  }

  private generateConstraintsForTypeAliasDeclaration(
    typeAlias: AST.TypeAliasDeclaration,
    env: Map<string, AST.Type>
  ): void {
    // ジェネリック型エイリアス情報を保存
    this.typeAliases.set(typeAlias.name, typeAlias)

    // 非ジェネリック型エイリアスの場合は従来通り環境に追加
    if (!typeAlias.typeParameters || typeAlias.typeParameters.length === 0) {
      env.set(typeAlias.name, typeAlias.aliasedType)
      this.currentEnvironment.set(typeAlias.name, typeAlias.aliasedType)
    }
    // ジェネリック型エイリアスは resolveTypeAlias で具体化時に処理
  }

  private createConstructorType(
    field: AST.TypeField,
    adtType: AST.Type
  ): AST.Type {
    if (field.type instanceof AST.PrimitiveType && field.type.name === "Unit") {
      // データなしのコンストラクタ (Red, Green, Blue)
      return adtType
    } else if (
      field.type instanceof AST.GenericType &&
      field.type.name === "Tuple"
    ) {
      // データ付きのコンストラクタ (RGB Int Int Int)
      let resultType = adtType

      // 型引数から逆順でカリー化された関数シグネチャを構築
      for (let i = field.type.typeArguments.length - 1; i >= 0; i--) {
        const paramType = field.type.typeArguments[i]
        resultType = new AST.FunctionType(
          paramType,
          resultType,
          field.line,
          field.column
        )
      }

      return resultType
    } else {
      // その他のケース（単一データ）
      return new AST.FunctionType(field.type, adtType, field.line, field.column)
    }
  }

  generateConstraintsForResolveExpression(
    resolveExpr: AST.ResolveExpression,
    env: Map<string, AST.Type>
  ): AST.Type {
    // resolve valueの型推論
    const valueType = this.generateConstraintsForExpression(
      resolveExpr.value,
      env
    )

    // 型引数が指定されている場合は制約を追加
    if (resolveExpr.typeArgument) {
      this.subtypeConstraints.push(
        new SubtypeConstraint(
          valueType,
          resolveExpr.typeArgument,
          resolveExpr.line,
          resolveExpr.column,
          "resolve type argument constraint"
        )
      )
    }

    // resolve式は関数型として扱う: () -> Promise<T>
    const promiseType = new AST.GenericType(
      "Promise",
      [resolveExpr.typeArgument || valueType],
      resolveExpr.line,
      resolveExpr.column
    )
    const unitType = new AST.PrimitiveType(
      "Unit",
      resolveExpr.line,
      resolveExpr.column
    )
    return new AST.FunctionType(
      unitType,
      promiseType,
      resolveExpr.line,
      resolveExpr.column
    )
  }

  generateConstraintsForRejectExpression(
    rejectExpr: AST.RejectExpression,
    env: Map<string, AST.Type>
  ): AST.Type {
    // reject valueの型推論（副作用としてnodeTypeMapへ登録）
    this.generateConstraintsForExpression(rejectExpr.value, env)

    // reject式は関数型として扱う: () -> Promise<Void>
    // 型引数が指定されている場合はそれを使用、なければVoidタイプ
    const rejectType =
      rejectExpr.typeArgument ||
      new AST.VoidType(rejectExpr.line, rejectExpr.column)
    const promiseType = new AST.GenericType(
      "Promise",
      [rejectType],
      rejectExpr.line,
      rejectExpr.column
    )
    const unitType = new AST.PrimitiveType(
      "Unit",
      rejectExpr.line,
      rejectExpr.column
    )
    return new AST.FunctionType(
      unitType,
      promiseType,
      rejectExpr.line,
      rejectExpr.column
    )
  }

  generateConstraintsForPromiseBlock(
    promiseBlock: AST.PromiseBlock,
    env: Map<string, AST.Type>
  ): AST.Type {
    // Promise block内の環境を作成（resolveとrejectが利用可能）
    const promiseEnv = new Map(env)

    // 型引数が明示的に指定されている場合はそれを使用
    let resolveType: AST.Type
    if (promiseBlock.typeArgument) {
      resolveType = promiseBlock.typeArgument
    } else {
      // 型推論用の型変数を作成
      resolveType = this.freshTypeVariable(
        promiseBlock.line,
        promiseBlock.column
      )
    }

    // Promise block内の文と式の型推論
    for (const stmt of promiseBlock.statements) {
      this.generateConstraintsForStatement(stmt, promiseEnv)
    }

    // returnExpressionがある場合はその型を推論
    if (promiseBlock.returnExpression) {
      // 型推論を実行（副作用としてnodeTypeMapへ登録）
      this.generateConstraintsForExpression(
        promiseBlock.returnExpression,
        promiseEnv
      )

      // 型引数が省略されている場合、resolve式から型を推論
      if (
        !promiseBlock.typeArgument &&
        promiseBlock.returnExpression.kind === "ResolveExpression"
      ) {
        const resolveExpr =
          promiseBlock.returnExpression as AST.ResolveExpression
        const valueType = this.generateConstraintsForExpression(
          resolveExpr.value,
          promiseEnv
        )
        resolveType = valueType
      }
    }

    // Promise<T>型を返す（() -> Promise<T>のラッパー型として）
    const promiseType = new AST.GenericType(
      "Promise",
      [resolveType],
      promiseBlock.line,
      promiseBlock.column
    )

    // 関数型でラップ（() -> Promise<T>）
    const unitType = new AST.PrimitiveType(
      "Unit",
      promiseBlock.line,
      promiseBlock.column
    )
    return new AST.FunctionType(
      unitType,
      promiseType,
      promiseBlock.line,
      promiseBlock.column
    )
  }

  public generateConstraintsForExpression(
    expr: AST.Expression,
    env: Map<string, AST.Type>,
    expectedType?: AST.Type
  ): AST.Type {
    let resultType: AST.Type

    switch (expr.kind) {
      case "Literal":
        resultType = this.generateConstraintsForLiteral(expr as AST.Literal)
        break

      case "Identifier":
        resultType = this.generateConstraintsForIdentifier(
          expr as AST.Identifier,
          env
        )
        break

      case "TemplateExpression":
        resultType = (this as any).generateConstraintsForTemplateExpression(
          expr as AST.TemplateExpression,
          env
        )
        break

      case "BinaryOperation":
        resultType = this.generateConstraintsForBinaryOperation(
          expr as AST.BinaryOperation,
          env
        )
        break

      case "NullishCoalescingExpression":
        resultType = this.generateConstraintsForNullishCoalescing(
          expr as AST.NullishCoalescingExpression,
          env
        )
        break

      case "UnaryOperation":
        resultType = this.generateConstraintsForUnaryOperation(
          expr as AST.UnaryOperation,
          env
        )
        break

      case "FunctionCall":
        resultType = this.generateConstraintsForFunctionCall(
          expr as AST.FunctionCall,
          env
        )
        break

      case "MethodCall":
        resultType = this.generateConstraintsForMethodCall(
          expr as AST.MethodCall,
          env
        )
        break

      case "BuiltinFunctionCall":
        resultType = this.generateConstraintsForBuiltinFunctionCall(
          expr as AST.BuiltinFunctionCall,
          env
        )
        break

      case "FunctionApplication":
        resultType = this.generateConstraintsForFunctionApplication(
          expr as AST.FunctionApplication,
          env
        )
        break

      case "Pipeline":
        resultType = this.generateConstraintsForPipeline(
          expr as AST.Pipeline,
          env
        )
        break

      case "ConditionalExpression":
        resultType = this.generateConstraintsForConditional(
          expr as AST.ConditionalExpression,
          env
        )
        break

      case "TernaryExpression":
        resultType = this.generateConstraintsForTernary(
          expr as AST.TernaryExpression,
          env,
          expectedType
        )
        break

      case "BlockExpression":
        resultType = this.generateConstraintsForBlockExpression(
          expr as AST.BlockExpression,
          env
        )
        break

      case "ConstructorExpression":
        resultType = this.generateConstraintsForConstructorExpression(
          expr as AST.ConstructorExpression,
          env
        )
        break
      case "SignalExpression":
        resultType = this.generateConstraintsForSignalExpression(
          expr as AST.SignalExpression,
          env
        )
        break
      case "AssignmentExpression":
        resultType = this.generateConstraintsForAssignmentExpression(
          expr as AST.AssignmentExpression,
          env
        )
        break

      case "FunctorMap":
        resultType = this.generateConstraintsForFunctorMap(
          expr as AST.FunctorMap,
          env
        )
        break

      case "ApplicativeApply":
        resultType = this.generateConstraintsForApplicativeApply(
          expr as AST.ApplicativeApply,
          env
        )
        break

      case "MonadBind":
        resultType = this.generateConstraintsForMonadBind(
          expr as AST.MonadBind,
          env
        )
        break

      case "RangeLiteral":
        resultType = this.generateConstraintsForRangeLiteral(
          expr as AST.RangeLiteral,
          env
        )
        break

      case "ListComprehension":
        resultType = this.generateConstraintsForListComprehension(
          expr as AST.ListComprehension,
          env
        )
        break

      case "ListComprehensionSugar":
        resultType = this.generateConstraintsForListComprehensionSugar(
          expr as AST.ListComprehensionSugar,
          env
        )
        break

      case "FunctionApplicationOperator":
        resultType = this.generateConstraintsForFunctionApplicationOperator(
          expr as AST.FunctionApplicationOperator,
          env
        )
        break

      case "LambdaExpression":
        resultType = this.generateConstraintsForLambdaExpression(
          expr as AST.LambdaExpression,
          env
        )
        break

      case "MatchExpression":
        resultType = this.generateConstraintsForMatchExpression(
          expr as AST.MatchExpression,
          env
        )
        break

      case "RecordExpression":
        resultType = this.generateConstraintsForRecordExpression(
          expr as AST.RecordExpression,
          env,
          expectedType
        )
        break

      case "RecordAccess":
        resultType = this.generateConstraintsForRecordAccess(
          expr as AST.RecordAccess,
          env
        )
        break

      case "ArrayLiteral":
        resultType = this.generateConstraintsForArrayLiteral(
          expr as AST.ArrayLiteral,
          env
        )
        break

      case "ArrayAccess":
        resultType = this.generateConstraintsForArrayAccess(
          expr as AST.ArrayAccess,
          env
        )
        break

      case "ListSugar":
        resultType = this.generateConstraintsForListSugar(
          expr as AST.ListSugar,
          env
        )
        break

      case "ConsExpression":
        resultType = this.generateConstraintsForConsExpression(
          expr as AST.ConsExpression,
          env
        )
        break

      case "TupleExpression":
        resultType = this.generateConstraintsForTupleExpression(
          expr as AST.TupleExpression,
          env
        )
        break

      case "StructExpression":
        resultType = this.generateConstraintsForStructExpression(
          expr as AST.StructExpression,
          env,
          expectedType
        )
        break

      case "SpreadExpression":
        resultType = this.generateConstraintsForSpreadExpression(
          expr as AST.SpreadExpression,
          env
        )
        break

      case "TryExpression":
        resultType = (this as any).generateConstraintsForTryExpression(
          expr as AST.TryExpression,
          env
        )
        break

      case "TypeAssertion":
        resultType = (this as any).generateConstraintsForTypeAssertion(
          expr as AST.TypeAssertion,
          env
        )
        break

      case "IsExpression":
        resultType = this.generateConstraintsForIsExpression(
          expr as AST.IsExpression,
          env
        )
        break

      case "PromiseBlock":
        resultType = this.generateConstraintsForPromiseBlock(
          expr as AST.PromiseBlock,
          env
        )
        break

      case "ResolveExpression":
        resultType = this.generateConstraintsForResolveExpression(
          expr as AST.ResolveExpression,
          env
        )
        break

      case "RejectExpression":
        resultType = this.generateConstraintsForRejectExpression(
          expr as AST.RejectExpression,
          env
        )
        break

      default:
        this.errors.push(
          new TypeInferenceError(
            `Unhandled expression type: ${expr.kind}`,
            expr.line,
            expr.column
          )
        )
        resultType = this.freshTypeVariable(expr.line, expr.column)
        break
    }

    // Track the type for this expression
    this.nodeTypeMap.set(expr, resultType)
    return resultType
  }

  private generateConstraintsForLiteral(literal: AST.Literal): AST.Type {
    switch (literal.literalType) {
      case "string":
        return new AST.PrimitiveType("String", literal.line, literal.column)
      case "integer":
        return new AST.PrimitiveType("Int", literal.line, literal.column)
      case "float":
        return new AST.PrimitiveType("Float", literal.line, literal.column)
      case "boolean":
        return new AST.PrimitiveType("Bool", literal.line, literal.column)
      case "unit":
        return new AST.PrimitiveType("Unit", literal.line, literal.column)
      default:
        return this.freshTypeVariable(literal.line, literal.column)
    }
  }

  private generateConstraintsForIdentifier(
    identifier: AST.Identifier,
    env: Map<string, AST.Type>
  ): AST.Type {
    // Debug: Log environment contents when looking for Red
    if (identifier.name === "Red") {
    }

    // Normal identifier lookup
    const type = env.get(identifier.name)
    if (!type) {
      this.errors.push(
        new TypeInferenceError(
          `Undefined variable: ${identifier.name}`,
          identifier.line,
          identifier.column
        )
      )
      return this.freshTypeVariable(identifier.line, identifier.column)
    }

    // Debug: Log the type we found
    if (identifier.name === "task1" || identifier.name === "taskFunc") {
      if (type.kind === "GenericType") {
        const gt = type as AST.GenericType
      }
    }

    // Debug logging for makeTuple function type resolution
    // if (identifier.name === 'makeTuple') {
    //   console.log(`DEBUG: makeTuple type resolution at ${identifier.line}:${identifier.column}`)
    //   console.log(`DEBUG: Original type from env:`, typeToStringUtil(type))
    //   const instantiated = this.instantiatePolymorphicType(type, identifier.line, identifier.column)
    //   console.log(`DEBUG: Instantiated type:`, typeToStringUtil(instantiated))
    //   return instantiated
    // }

    // Instantiate polymorphic types when looking up from environment
    return this.instantiatePolymorphicType(
      type,
      identifier.line,
      identifier.column
    )
  }

  private generateConstraintsForBinaryOperation(
    binOp: AST.BinaryOperation,
    env: Map<string, AST.Type>
  ): AST.Type {
    const leftType = this.generateConstraintsForExpression(binOp.left, env)
    const rightType = this.generateConstraintsForExpression(binOp.right, env)

    // left/rightの式に型情報を設定
    binOp.left.type = leftType
    binOp.right.type = rightType

    switch (binOp.operator) {
      case "+": {
        // 構造体の演算子オーバーロードをチェック
        const plusOperatorDef = this.findOperatorDefinition(leftType, "+", env)
        if (plusOperatorDef) {
          // 演算子定義が見つかった場合、その戻り値型を使用
          return plusOperatorDef.returnType
        }

        // 演算子定義がない場合は従来の処理
        // + 演算子は数値演算か文字列結合のどちらか
        // 左右のオペランドが同じ型である制約
        this.addConstraint(
          new TypeConstraint(
            leftType,
            rightType,
            binOp.line,
            binOp.column,
            `Binary operation + operands must have same type`
          )
        )

        // 結果の型は左のオペランドと同じ型
        return leftType
      }

      case "-":
      case "*":
      case "/":
      case "%":
      case "**": {
        // 構造体の演算子オーバーロードをチェック
        const operatorDef = this.findOperatorDefinition(
          leftType,
          binOp.operator,
          env
        )
        if (operatorDef) {
          // 演算子定義が見つかった場合、その戻り値型を使用
          return operatorDef.returnType
        }

        // 演算子定義がない場合は従来の処理
        // 数値演算: 両オペランドは同じ型でなければならず、結果も同じ型
        this.addConstraint(
          new TypeConstraint(
            leftType,
            rightType,
            binOp.line,
            binOp.column,
            `Binary operation ${binOp.operator} operands must have same type`
          )
        )

        // 結果の型は左のオペランドと同じ型
        return leftType
      }

      case "==":
      case "!=":
      case "<":
      case ">":
      case "<=":
      case ">=":
        // 比較演算: 両オペランドは同じ型、結果はBool
        this.addConstraint(
          new TypeConstraint(
            leftType,
            rightType,
            binOp.line,
            binOp.column,
            `Comparison ${binOp.operator} operands must match`
          )
        )
        return new AST.PrimitiveType("Bool", binOp.line, binOp.column)

      case "&&":
      case "||": {
        // 論理演算: 基本的にはBool、ただし構造体のオーバーロードも考慮
        const hasStructTypeLogical =
          this.isStructOrResolvesToStruct(leftType, env) ||
          this.isStructOrResolvesToStruct(rightType, env)

        if (hasStructTypeLogical) {
          // 構造体のオーバーロードが考えられる場合: 左右のオペランドが同じ型である制約のみ
          this.addConstraint(
            new TypeConstraint(
              leftType,
              rightType,
              binOp.line,
              binOp.column,
              `Logical operation ${binOp.operator} operands must have same type (struct overload)`
            )
          )
          // 結果の型は演算子オーバーロードによって決まるが、一般的にはBoolを返す
          // 構造体オーバーロードでは通常Bool型を返すことが多い
          return new AST.PrimitiveType("Bool", binOp.line, binOp.column)
        } else {
          // 通常の論理演算: 両オペランドはBool、結果もBool
          const boolType = new AST.PrimitiveType(
            "Bool",
            binOp.line,
            binOp.column
          )
          this.addConstraint(
            new TypeConstraint(
              leftType,
              boolType,
              binOp.left.line,
              binOp.left.column,
              `Logical operation ${binOp.operator} left operand`
            )
          )
          this.addConstraint(
            new TypeConstraint(
              rightType,
              boolType,
              binOp.right.line,
              binOp.right.column,
              `Logical operation ${binOp.operator} right operand`
            )
          )
          return boolType
        }
      }

      case ":": {
        // CONS演算子: a : List<a> -> List<a>
        // 左オペランドは要素、右オペランドはリスト、結果は同じ要素型のリスト
        const expectedListType = new AST.GenericType(
          "List",
          [leftType],
          binOp.right.line,
          binOp.right.column
        )

        // 右オペランドはList<leftType>型でなければならない
        this.addConstraint(
          new TypeConstraint(
            rightType,
            expectedListType,
            binOp.right.line,
            binOp.right.column,
            `CONS operator (:) right operand must be List<${typeToStringUtil(leftType)}>`
          )
        )

        // 結果の型もList<leftType>
        return expectedListType
      }

      case ":=": {
        // Signal代入演算子: Signal<a> := a -> Signal<a>
        // 左オペランドはSignal<T>、右オペランドはT、結果はSignal<T>
        if (leftType.kind === "GenericType") {
          const genType = leftType as AST.GenericType
          if (genType.name === "Signal" && genType.typeArguments.length === 1) {
            const signalValueType = genType.typeArguments[0]

            // 右オペランドはSignalの値型と一致する必要がある
            this.addConstraint(
              new TypeConstraint(
                rightType,
                signalValueType,
                binOp.right.line,
                binOp.right.column,
                `Signal assignment (:=) value must match Signal type ${typeToStringUtil(signalValueType)}`
              )
            )

            // 結果は左オペランドと同じSignal<T>型
            return leftType
          }
        }

        // 左オペランドがSignal型でない場合はエラー
        this.errors.push(
          new TypeInferenceError(
            `Signal assignment (:=) can only be applied to Signal types, but got ${typeToStringUtil(leftType)}`,
            binOp.left.line,
            binOp.left.column
          )
        )
        return this.freshTypeVariable(binOp.line, binOp.column)
      }

      default:
        this.errors.push(
          new TypeInferenceError(
            `Unknown binary operator: ${binOp.operator}`,
            binOp.line,
            binOp.column
          )
        )
        return this.freshTypeVariable(binOp.line, binOp.column)
    }
  }

  private generateConstraintsForNullishCoalescing(
    nullishCoalescing: AST.NullishCoalescingExpression,
    env: Map<string, AST.Type>
  ): AST.Type {
    const leftType = this.generateConstraintsForExpression(
      nullishCoalescing.left,
      env
    )
    const rightType = this.generateConstraintsForExpression(
      nullishCoalescing.right,
      env
    )

    // 左辺と右辺の式に型情報を設定
    nullishCoalescing.left.type = leftType
    nullishCoalescing.right.type = rightType

    // 型変数の場合は、Maybe型の可能性を考慮した特別な制約を追加
    if (leftType.kind === "TypeVariable") {
      // 右辺も型変数の場合は、より柔軟な制約を追加
      if (rightType.kind === "TypeVariable") {
        const resultTypeVar = this.freshTypeVariable(
          nullishCoalescing.line,
          nullishCoalescing.column
        )

        // 左辺がMaybe<resultTypeVar>または右辺がresultTypeVarである制約
        const maybeType = new AST.GenericType(
          "Maybe",
          [resultTypeVar],
          nullishCoalescing.line,
          nullishCoalescing.column
        )

        // 左辺型をMaybe型として制約
        this.addConstraint(
          new TypeConstraint(
            leftType,
            maybeType,
            nullishCoalescing.line,
            nullishCoalescing.column,
            "Nullish coalescing left operand should be Maybe type"
          )
        )

        // 右辺型は結果型と互換
        this.addConstraint(
          new TypeConstraint(
            rightType,
            resultTypeVar,
            nullishCoalescing.line,
            nullishCoalescing.column,
            "Nullish coalescing right operand should be compatible with result"
          )
        )

        return resultTypeVar
      }

      // 右辺もMaybe型の場合は特別な処理
      if (isMaybeTypeUtil(rightType)) {
        const rightMaybeInnerType = (rightType as AST.GenericType)
          .typeArguments[0]

        // 左辺がMaybe型である可能性を考慮
        const innerTypeVar = this.freshTypeVariable(
          nullishCoalescing.line,
          nullishCoalescing.column
        )
        const maybeType = new AST.GenericType(
          "Maybe",
          [innerTypeVar],
          nullishCoalescing.line,
          nullishCoalescing.column
        )

        // 左辺型をMaybe型として制約
        this.addConstraint(
          new TypeConstraint(
            leftType,
            maybeType,
            nullishCoalescing.line,
            nullishCoalescing.column,
            "Nullish coalescing left operand should be Maybe type"
          )
        )

        // 両方の内部型の互換性制約
        this.addConstraint(
          new TypeConstraint(
            innerTypeVar,
            rightMaybeInnerType,
            nullishCoalescing.line,
            nullishCoalescing.column,
            "Maybe inner types should be compatible"
          )
        )

        return innerTypeVar
      }

      // 右辺が非Maybe型の場合
      const innerTypeVar = this.freshTypeVariable(
        nullishCoalescing.line,
        nullishCoalescing.column
      )
      const maybeType = new AST.GenericType(
        "Maybe",
        [innerTypeVar],
        nullishCoalescing.line,
        nullishCoalescing.column
      )

      // 左辺型をMaybe型として制約
      this.addConstraint(
        new TypeConstraint(
          leftType,
          maybeType,
          nullishCoalescing.line,
          nullishCoalescing.column,
          "Nullish coalescing left operand should be Maybe type"
        )
      )

      // 内部型と右辺型の互換性制約
      this.addConstraint(
        new TypeConstraint(
          innerTypeVar,
          rightType,
          nullishCoalescing.line,
          nullishCoalescing.column,
          "Maybe inner type should be compatible with default value"
        )
      )

      return innerTypeVar
    }

    // Maybe型の場合: Maybe<T> ?? U => T | U (実際にはTが返される)
    if (isMaybeTypeUtil(leftType)) {
      const maybeInnerType = (leftType as AST.GenericType).typeArguments[0]

      // 右辺もMaybe型の場合: Maybe<T> ?? Maybe<U> => T | U
      if (isMaybeTypeUtil(rightType)) {
        const rightMaybeInnerType = (rightType as AST.GenericType)
          .typeArguments[0]
        // 両方の内部型の互換性をチェック
        this.addConstraint(
          new TypeConstraint(
            maybeInnerType,
            rightMaybeInnerType,
            nullishCoalescing.line,
            nullishCoalescing.column,
            "Maybe inner types should be compatible"
          )
        )
        return maybeInnerType
      }

      // 内部型と右辺型の互換性をチェック
      this.addConstraint(
        new TypeConstraint(
          maybeInnerType,
          rightType,
          nullishCoalescing.line,
          nullishCoalescing.column,
          "Maybe inner type should be compatible with default value"
        )
      )
      return maybeInnerType
    }

    // Either型の場合: Either<L, R> ?? U => R | U (実際にはRが返される)
    if (isEitherTypeUtil(leftType)) {
      const rightTypeFromEither = (leftType as AST.GenericType).typeArguments[1]

      // 右辺もEither型の場合: Either<L1, R1> ?? Either<L2, R2> => R1 | R2
      if (rightType.kind === "GenericType" && rightType.name === "Either") {
        const rightEitherRightType = (rightType as AST.GenericType)
          .typeArguments[1]
        // 両方のRight型の互換性をチェック
        this.addConstraint(
          new TypeConstraint(
            rightTypeFromEither,
            rightEitherRightType,
            nullishCoalescing.line,
            nullishCoalescing.column,
            "Either Right types should be compatible"
          )
        )
        return rightTypeFromEither
      }

      // Right型と右辺型の互換性をチェック
      this.addConstraint(
        new TypeConstraint(
          rightTypeFromEither,
          rightType,
          nullishCoalescing.line,
          nullishCoalescing.column,
          "Either Right type should be compatible with default value"
        )
      )
      return rightTypeFromEither
    }

    // その他の場合: 通常のnull合体演算子として扱う（TypeScript風）
    // 左辺と右辺の型が互換である必要がある
    this.addConstraint(
      new TypeConstraint(
        leftType,
        rightType,
        nullishCoalescing.line,
        nullishCoalescing.column,
        "?? operands should have compatible types for non-Maybe/Either types"
      )
    )

    // TypeScript風の場合、より寛容な型を返す（通常は右辺の型）
    return rightType
  }

  private generateConstraintsForUnaryOperation(
    unaryOp: AST.UnaryOperation,
    env: Map<string, AST.Type>
  ): AST.Type {
    const operandType = this.generateConstraintsForExpression(
      unaryOp.operand,
      env
    )

    switch (unaryOp.operator) {
      case "-": {
        // 数値の単項マイナス: Int -> Int, Float -> Float
        const intType = new AST.PrimitiveType(
          "Int",
          unaryOp.line,
          unaryOp.column
        )
        const _floatType = new AST.PrimitiveType(
          "Float",
          unaryOp.line,
          unaryOp.column
        )

        // まず Int 型として制約を追加
        this.addConstraint(
          new TypeConstraint(
            operandType,
            intType,
            unaryOp.operand.line,
            unaryOp.operand.column,
            `Unary minus operand (Int)`
          )
        )
        return intType
      }

      case "!": {
        // 論理否定: Bool -> Bool
        const boolType = new AST.PrimitiveType(
          "Bool",
          unaryOp.line,
          unaryOp.column
        )
        this.addConstraint(
          new TypeConstraint(
            operandType,
            boolType,
            unaryOp.operand.line,
            unaryOp.operand.column,
            `Logical negation operand`
          )
        )
        return boolType
      }

      case "*": {
        // Signal値取得: Signal<T> -> T
        // operandTypeがSignal<T>の場合、Tを返す
        if (operandType.kind === "GenericType") {
          const genType = operandType as AST.GenericType
          if (genType.name === "Signal" && genType.typeArguments.length === 1) {
            return genType.typeArguments[0] // Signal<T> -> T
          }
        }

        // Signal型でない場合はエラー
        this.errors.push(
          new TypeInferenceError(
            `getValue operator (*) can only be applied to Signal types, got ${typeToStringUtil(operandType)}`,
            unaryOp.operand.line,
            unaryOp.operand.column
          )
        )
        return this.freshTypeVariable(unaryOp.line, unaryOp.column)
      }

      default:
        this.errors.push(
          new TypeInferenceError(
            `Unknown unary operator: ${unaryOp.operator}`,
            unaryOp.line,
            unaryOp.column
          )
        )
        return this.freshTypeVariable(unaryOp.line, unaryOp.column)
    }
  }

  private generateConstraintsForFunctionCall(
    call: AST.FunctionCall,
    env: Map<string, AST.Type>
  ): AST.Type {
    if (call.function.kind === "Identifier") {
      const funcName = (call.function as AST.Identifier).name
    }
    // print/putStrLn/show関数の特別処理
    if (call.function.kind === "Identifier") {
      const funcName = (call.function as AST.Identifier).name
      if (
        (funcName === "print" ||
          funcName === "putStrLn" ||
          funcName === "show") &&
        call.arguments.length === 1
      ) {
        // print/putStrLn/show関数は任意の型を受け取り、Unit型を返す
        this.generateConstraintsForExpression(call.arguments[0], env)
        return new AST.PrimitiveType("Unit", call.line, call.column)
      }

      // tryRun関数の特別処理
      if (
        (funcName === "tryRun" || funcName === "ssrgTryRun") &&
        call.arguments.length === 1
      ) {
        // 引数の型を推論
        const argType = this.generateConstraintsForExpression(
          call.arguments[0],
          env
        )

        // エラー型を決定（型引数があれば使用、なければString）
        let errorType: AST.Type
        if (call.typeArguments && call.typeArguments.length > 0) {
          errorType = call.typeArguments[0]
        } else {
          errorType = new AST.PrimitiveType("String", call.line, call.column)
        }

        // Task<T> から T を取得
        let valueType: AST.Type
        if (
          argType.kind === "GenericType" &&
          (argType as AST.GenericType).name === "Task"
        ) {
          const taskType = argType as AST.GenericType
          valueType = taskType.typeArguments[0]
        } else {
          // エラー: 引数はTask型でなければならない
          valueType = this.freshTypeVariable(call.line, call.column)
        }

        // Promise<Either<ErrorType, T>> を返す
        const eitherType = new AST.GenericType(
          "Either",
          [errorType, valueType],
          call.line,
          call.column
        )
        return new AST.GenericType(
          "Promise",
          [eitherType],
          call.line,
          call.column
        )
      }
    }

    // 明示的型引数がある場合と無い場合で処理を分ける
    let funcType: AST.Type
    let resultType: AST.Type

    if (call.typeArguments && call.typeArguments.length > 0) {
      // 明示的型引数がある場合は、環境から直接多相型を取得
      if (call.function.kind === "Identifier") {
        const identifier = call.function as AST.Identifier
        const rawFuncType = env.get(identifier.name)
        if (!rawFuncType) {
          throw new Error(`Undefined function: ${identifier.name}`)
        }

        resultType = this.instantiateWithExplicitTypeArguments(
          rawFuncType,
          call.typeArguments,
          call.line,
          call.column
        )
        funcType = resultType
      } else {
        // 複雑な式の場合はとりあえず従来の方法
        funcType = this.generateConstraintsForExpression(call.function, env)
        resultType = this.instantiateWithExplicitTypeArguments(
          funcType,
          call.typeArguments,
          call.line,
          call.column
        )
      }
    } else {
      // 従来の処理
      // 関数が識別子の場合、環境から直接型を取得し、適切にインスタンス化
      if (call.function.kind === "Identifier") {
        const identifier = call.function as AST.Identifier
        const rawFuncType = env.get(identifier.name)
        if (rawFuncType) {
          // 多相型のインスタンス化を確実に実行
          resultType = this.instantiatePolymorphicType(
            rawFuncType,
            call.line,
            call.column
          )
          funcType = resultType
        } else {
          funcType = this.generateConstraintsForExpression(call.function, env)
          resultType = this.instantiatePolymorphicType(
            funcType,
            call.line,
            call.column
          )
        }
      } else {
        funcType = this.generateConstraintsForExpression(call.function, env)
        resultType = this.instantiatePolymorphicType(
          funcType,
          call.line,
          call.column
        )
      }

      // 引数が0個の場合は、関数がユニット型を取る関数として扱う
      if (call.arguments.length === 0) {
        // 関数シグネチャが既知の場合、その戻り値型を抽出
        if (funcType.kind === "FunctionType") {
          const ft = funcType as AST.FunctionType
          // Unit -> ReturnType の形を期待
          const expectedFuncType = new AST.FunctionType(
            new AST.PrimitiveType("Unit", call.line, call.column),
            ft.returnType, // 既存の戻り値型を使用
            call.line,
            call.column
          )

          this.addConstraint(
            new TypeConstraint(
              funcType,
              expectedFuncType,
              call.line,
              call.column,
              `Unit function application`
            )
          )

          return ft.returnType // 戻り値型を直接返す
        }

        // 関数シグネチャが不明な場合のフォールバック
        const result = this.freshTypeVariable(call.line, call.column)
        const expectedFuncType = new AST.FunctionType(
          new AST.PrimitiveType("Unit", call.line, call.column),
          result,
          call.line,
          call.column
        )

        this.addConstraint(
          new TypeConstraint(
            funcType,
            expectedFuncType,
            call.line,
            call.column,
            `Unit function application`
          )
        )

        return result
      }
    }

    // 各引数に対して関数適用の制約を生成
    for (const arg of call.arguments) {
      let expectedParamType: AST.Type = this.freshTypeVariable(
        call.line,
        call.column
      )

      if (call.function.kind === "Identifier") {
        const funcName = (call.function as AST.Identifier).name
        const funcTypeFromEnv = env.get(funcName)
        if (funcName === "p" || funcName === "div") {
          if (funcTypeFromEnv) {
          }
        }
      }

      // 関数型が既知の場合、パラメータ型を抽出
      if (resultType.kind === "FunctionType") {
        const funcTypeInstance = resultType as AST.FunctionType
        expectedParamType = funcTypeInstance.paramType
      } else {
      }

      const actualArgType = this.generateConstraintsForExpression(
        arg,
        env,
        expectedParamType
      )
      const newResultType = this.freshTypeVariable(call.line, call.column)

      // 現在の結果型は expectedParamType から newResultType へのFunction型でなければならない
      const expectedFuncType = new AST.FunctionType(
        expectedParamType,
        newResultType,
        call.line,
        call.column
      )

      this.addConstraint(
        new TypeConstraint(
          resultType,
          expectedFuncType,
          call.line,
          call.column,
          `Function application structure`
        )
      )

      // 常に統一制約を使用（一旦元に戻す）
      this.addConstraint(
        new TypeConstraint(
          actualArgType,
          expectedParamType,
          call.line,
          call.column,
          `Function parameter type: ${typeToStringUtil(actualArgType)} ~ ${typeToStringUtil(expectedParamType)}`
        )
      )

      resultType = newResultType
    }

    return resultType
  }

  private generateConstraintsForBuiltinFunctionCall(
    call: AST.BuiltinFunctionCall,
    env: Map<string, AST.Type>
  ): AST.Type {
    switch (call.functionName) {
      case "print":
      case "putStrLn":
      case "show":
        // Type: 'a -> Unit (polymorphic)
        if (call.arguments.length === 1) {
          // Just check that the argument has some type, but we accept anything
          this.generateConstraintsForExpression(call.arguments[0], env)
        }
        return new AST.PrimitiveType("Unit", call.line, call.column)

      case "toString":
        // Type: 'a -> String (polymorphic)
        if (call.arguments.length === 1) {
          // Just check that the argument has some type, but we accept anything
          this.generateConstraintsForExpression(call.arguments[0], env)
        }
        return new AST.PrimitiveType("String", call.line, call.column)

      case "head":
        // Type: List<T> -> Maybe<T>
        if (call.arguments.length === 1) {
          const listType = this.generateConstraintsForExpression(
            call.arguments[0],
            env
          )
          const elementType = this.freshTypeVariable(call.line, call.column)

          // 引数はList<T>型でなければならない
          const expectedListType = new AST.GenericType(
            "List",
            [elementType],
            call.line,
            call.column
          )
          this.addConstraint(
            new TypeConstraint(
              listType,
              expectedListType,
              call.line,
              call.column,
              "head function requires List<T> argument"
            )
          )

          // 戻り値はMaybe<T>型
          return new AST.GenericType(
            "Maybe",
            [elementType],
            call.line,
            call.column
          )
        }
        throw new Error("head function requires exactly one argument")

      case "tail":
        // Type: List<T> -> List<T>
        if (call.arguments.length === 1) {
          const listType = this.generateConstraintsForExpression(
            call.arguments[0],
            env
          )
          const elementType = this.freshTypeVariable(call.line, call.column)

          // 引数はList<T>型でなければならない
          const expectedListType = new AST.GenericType(
            "List",
            [elementType],
            call.line,
            call.column
          )
          this.addConstraint(
            new TypeConstraint(
              listType,
              expectedListType,
              call.line,
              call.column,
              "tail function requires List<T> argument"
            )
          )

          // 戻り値も同じList<T>型
          return expectedListType
        }
        throw new Error("tail function requires exactly one argument")

      case "toInt":
        // Type: 'a -> Int
        if (call.arguments.length === 1) {
          // 引数の型をチェックするが、何でも受け付ける
          this.generateConstraintsForExpression(call.arguments[0], env)
          return new AST.PrimitiveType("Int", call.line, call.column)
        }
        throw new Error("toInt function requires exactly one argument")

      case "toFloat":
        // Type: 'a -> Float
        if (call.arguments.length === 1) {
          // 引数の型をチェックするが、何でも受け付ける
          this.generateConstraintsForExpression(call.arguments[0], env)
          return new AST.PrimitiveType("Float", call.line, call.column)
        }
        throw new Error("toFloat function requires exactly one argument")

      case "typeof":
        // Type: 'a -> String (polymorphic)
        if (call.arguments.length === 1) {
          // 引数の型をチェックするが、何でも受け付ける
          this.generateConstraintsForExpression(call.arguments[0], env)
          return new AST.PrimitiveType("String", call.line, call.column)
        }
        throw new Error("typeof function requires exactly one argument")

      case "subscribe":
        // Type: Signal<T> -> (T -> Unit) -> String
        if (call.arguments.length === 2) {
          const signalType = this.generateConstraintsForExpression(
            call.arguments[0],
            env
          )
          const observerType = this.generateConstraintsForExpression(
            call.arguments[1],
            env
          )

          const valueType = this.freshTypeVariable(call.line, call.column)
          const expectedSignalType = new AST.GenericType(
            "Signal",
            [valueType],
            call.line,
            call.column
          )
          const expectedObserverType = new AST.FunctionType(
            valueType,
            new AST.PrimitiveType("Unit", call.line, call.column),
            call.line,
            call.column
          )

          this.addConstraint(
            new TypeConstraint(
              signalType,
              expectedSignalType,
              call.line,
              call.column,
              "subscribe requires Signal<T> as first argument"
            )
          )
          this.addConstraint(
            new TypeConstraint(
              observerType,
              expectedObserverType,
              call.line,
              call.column,
              "subscribe requires (T -> Unit) observer function as second argument"
            )
          )

          return new AST.PrimitiveType("String", call.line, call.column)
        }
        throw new Error(
          "subscribe function requires exactly two arguments: signal and observer"
        )

      case "unsubscribe":
        // Type: String -> Unit
        if (call.arguments.length === 1) {
          const keyType = this.generateConstraintsForExpression(
            call.arguments[0],
            env
          )
          this.addConstraint(
            new TypeConstraint(
              keyType,
              new AST.PrimitiveType("String", call.line, call.column),
              call.line,
              call.column,
              "unsubscribe requires String subscription key"
            )
          )
          return new AST.PrimitiveType("Unit", call.line, call.column)
        }
        throw new Error(
          "unsubscribe function requires exactly one argument: subscription key"
        )

      case "detach":
        // Type: Signal<T> -> Unit
        if (call.arguments.length === 1) {
          const signalType = this.generateConstraintsForExpression(
            call.arguments[0],
            env
          )
          const valueType = this.freshTypeVariable(call.line, call.column)
          const expectedSignalType = new AST.GenericType(
            "Signal",
            [valueType],
            call.line,
            call.column
          )

          this.addConstraint(
            new TypeConstraint(
              signalType,
              expectedSignalType,
              call.line,
              call.column,
              "detach requires Signal<T> argument"
            )
          )

          return new AST.PrimitiveType("Unit", call.line, call.column)
        }
        throw new Error("detach function requires exactly one argument: signal")

      default:
        this.errors.push(
          new TypeInferenceError(
            `Unknown builtin function: ${call.functionName}`,
            call.line,
            call.column
          )
        )
        return this.freshTypeVariable(call.line, call.column)
    }
  }

  private generateConstraintsForIsExpression(
    isExpr: AST.IsExpression,
    env: Map<string, AST.Type>
  ): AST.Type {
    // is式は常にBool型を返す
    // Type: 'a -> Type -> Bool

    // 左辺の式の型を推論
    this.generateConstraintsForExpression(isExpr.left, env)

    // 右辺の型は型注釈なので型推論は不要
    // 実際の型等価性チェックは実行時に行われる

    return new AST.PrimitiveType("Bool", isExpr.line, isExpr.column)
  }

  private generateConstraintsForFunctionApplication(
    app: AST.FunctionApplication,
    env: Map<string, AST.Type>
  ): AST.Type {
    if (app.function.kind === "Identifier") {
      const funcName = (app.function as AST.Identifier).name
    }

    // 関数型を取得（環境から直接取得を優先）
    let funcType: AST.Type
    if (app.function.kind === "Identifier") {
      const identifier = app.function as AST.Identifier
      const rawFuncType = env.get(identifier.name)
      if (rawFuncType) {
        funcType = this.instantiatePolymorphicType(
          rawFuncType,
          app.line,
          app.column
        )
      } else {
        funcType = this.generateConstraintsForExpression(app.function, env)
      }
    } else {
      funcType = this.generateConstraintsForExpression(app.function, env)
    }

    // 期待されるパラメータ型を抽出
    let expectedParamType: AST.Type
    if (funcType.kind === "FunctionType") {
      const funcTypeInstance = funcType as AST.FunctionType
      expectedParamType = funcTypeInstance.paramType
    } else {
      expectedParamType = this.freshTypeVariable(app.line, app.column)
    }

    const argType = this.generateConstraintsForExpression(
      app.argument,
      env,
      expectedParamType
    )
    const resultType = this.freshTypeVariable(app.line, app.column)

    // 関数型が既知の場合は、その戻り値型を使用
    if (funcType.kind === "FunctionType") {
      const funcTypeInstance = funcType as AST.FunctionType
      // 引数型の制約を追加
      this.addConstraint(
        new TypeConstraint(
          argType,
          expectedParamType,
          app.line,
          app.column,
          `Function application argument type`
        )
      )
      return funcTypeInstance.returnType
    } else {
      // Function型は expectedParamType から resultType への関数でなければならない
      const expectedFuncType = new AST.FunctionType(
        expectedParamType,
        resultType,
        app.line,
        app.column
      )

      this.addConstraint(
        new TypeConstraint(
          funcType,
          expectedFuncType,
          app.line,
          app.column,
          `Function application structure`
        )
      )

      // 常に統一制約を使用（一旦元に戻す）
      this.addConstraint(
        new TypeConstraint(
          argType,
          expectedParamType,
          app.line,
          app.column,
          `Function application parameter type: ${typeToStringUtil(argType)} ~ ${typeToStringUtil(expectedParamType)}`
        )
      )

      return resultType
    }
  }

  private generateConstraintsForPipeline(
    pipe: AST.Pipeline,
    env: Map<string, AST.Type>
  ): AST.Type {
    const leftType = this.generateConstraintsForExpression(pipe.left, env)
    const rightType = this.generateConstraintsForExpression(pipe.right, env)
    const resultType = this.freshTypeVariable(pipe.line, pipe.column)

    // 右側は左側の型から結果型への関数でなければならない
    const expectedFuncType = new AST.FunctionType(
      leftType,
      resultType,
      pipe.line,
      pipe.column
    )

    this.addConstraint(
      new TypeConstraint(
        rightType,
        expectedFuncType,
        pipe.line,
        pipe.column,
        `Pipeline operator`
      )
    )

    return resultType
  }

  private generateConstraintsForConditional(
    cond: AST.ConditionalExpression,
    env: Map<string, AST.Type>
  ): AST.Type {
    const condType = this.generateConstraintsForExpression(cond.condition, env)
    const thenType = this.generateConstraintsForExpression(
      cond.thenExpression,
      env
    )
    const elseType = this.generateConstraintsForExpression(
      cond.elseExpression,
      env
    )

    // 条件はBool型でなければならない
    this.addConstraint(
      new TypeConstraint(
        condType,
        new AST.PrimitiveType(
          "Bool",
          cond.condition.line,
          cond.condition.column
        ),
        cond.condition.line,
        cond.condition.column,
        `Conditional expression condition`
      )
    )

    // thenとelseの型が同じかチェック
    if (typesEqualUtil(thenType, elseType)) {
      // 同じ型の場合はそのまま返す
      return thenType
    }

    // 異なる型の場合は、ユニオン型として返す
    return createFlattenedUnionTypeUtil(
      [thenType, elseType],
      cond.line,
      cond.column
    )
  }

  // 型統一を厳密にチェックする（Union型の特別ケースを除外）
  private canUnifyWithoutUnion(type1: AST.Type, type2: AST.Type): boolean {
    try {
      // 型が完全に一致する場合
      if (typesEqualUtil(type1, type2)) {
        return true
      }

      // 型変数の場合は統一可能
      if (type1.kind === "TypeVariable" || type2.kind === "TypeVariable") {
        return true
      }

      // 基本型の場合は名前が一致する必要がある
      if (type1.kind === "PrimitiveType" && type2.kind === "PrimitiveType") {
        return type1.name === type2.name
      }

      // Union型の場合は統一しない（厳密モード）
      if (type1.kind === "UnionType" || type2.kind === "UnionType") {
        return false
      }

      // 構造的な型の場合（ジェネリック型など）
      if (type1.kind === "GenericType" && type2.kind === "GenericType") {
        const param1 = type1 as AST.GenericType
        const param2 = type2 as AST.GenericType

        // 同じ型コンストラクタの場合
        if (
          param1.name === param2.name &&
          param1.typeArguments.length === param2.typeArguments.length
        ) {
          // 各型引数が統一可能かチェック
          for (let i = 0; i < param1.typeArguments.length; i++) {
            if (
              !this.canUnifyWithoutUnion(
                param1.typeArguments[i],
                param2.typeArguments[i]
              )
            ) {
              return false
            }
          }
          return true
        }
        return false
      }

      // 関数型の場合
      if (type1.kind === "FunctionType" && type2.kind === "FunctionType") {
        const func1 = type1 as AST.FunctionType
        const func2 = type2 as AST.FunctionType
        return (
          this.canUnifyWithoutUnion(func1.paramType, func2.paramType) &&
          this.canUnifyWithoutUnion(func1.returnType, func2.returnType)
        )
      }

      // その他の複雑な型の場合は通常の統一を試みる
      // ただし、実際の制約は追加せずに統一可能性のみチェック
      this.unify(type1, type2)
      return true
    } catch {
      return false
    }
  }

  private generateConstraintsForTernary(
    ternary: AST.TernaryExpression,
    env: Map<string, AST.Type>,
    expectedType?: AST.Type
  ): AST.Type {
    const condType = this.generateConstraintsForExpression(
      ternary.condition,
      env
    )
    const trueType = this.generateConstraintsForExpression(
      ternary.trueExpression,
      env,
      expectedType
    )
    const falseType = this.generateConstraintsForExpression(
      ternary.falseExpression,
      env,
      expectedType
    )

    // 条件はBool型でなければならない
    this.addConstraint(
      new TypeConstraint(
        condType,
        new AST.PrimitiveType(
          "Bool",
          ternary.condition.line,
          ternary.condition.column
        ),
        ternary.condition.line,
        ternary.condition.column,
        `Ternary expression condition`
      )
    )

    // trueとfalseの型が同じかチェック
    if (typesEqualUtil(trueType, falseType)) {
      // 同じ型の場合はそのまま返す
      return trueType
    }

    // 期待される型が指定されている場合、各分岐が期待される型と一致するかチェック
    if (expectedType) {
      // 各分岐が期待される型と統一できるかを個別にチェック
      // Union型の場合は、単純な型変換ではなく、型の互換性を厳密にチェックする

      // true分岐の型チェック
      if (!this.canUnifyWithoutUnion(trueType, expectedType)) {
        this.errors.push(
          new TypeInferenceError(
            `Ternary true branch type ${typeToStringUtil(trueType)} cannot be assigned to expected type ${typeToStringUtil(expectedType)}`,
            ternary.trueExpression.line,
            ternary.trueExpression.column
          )
        )
      }

      // false分岐の型チェック
      if (!this.canUnifyWithoutUnion(falseType, expectedType)) {
        this.errors.push(
          new TypeInferenceError(
            `Ternary false branch type ${typeToStringUtil(falseType)} cannot be assigned to expected type ${typeToStringUtil(expectedType)}`,
            ternary.falseExpression.line,
            ternary.falseExpression.column
          )
        )
      }

      // 制約を追加して期待される型を返す
      this.addConstraint(
        new TypeConstraint(
          trueType,
          expectedType,
          ternary.trueExpression.line,
          ternary.trueExpression.column,
          `Ternary true branch type`
        )
      )
      this.addConstraint(
        new TypeConstraint(
          falseType,
          expectedType,
          ternary.falseExpression.line,
          ternary.falseExpression.column,
          `Ternary false branch type`
        )
      )
      return expectedType
    }

    // 期待される型が指定されていない場合は、ユニオン型として返す
    return createFlattenedUnionTypeUtil(
      [trueType, falseType],
      ternary.line,
      ternary.column
    )
  }

  private generateConstraintsForBlockExpression(
    block: AST.BlockExpression,
    env: Map<string, AST.Type>
  ): AST.Type {
    // Create a new environment for the block scope
    const blockEnv = new Map(env)

    // Process all statements in the block
    for (const statement of block.statements) {
      this.generateConstraintsForStatement(statement, blockEnv)
    }

    // The block's type is determined by its return expression
    if (block.returnExpression) {
      return this.generateConstraintsForExpression(
        block.returnExpression,
        blockEnv
      )
    }

    // If no explicit return expression, check if the last statement is an expression
    if (block.statements.length > 0) {
      const lastStatement = block.statements[block.statements.length - 1]
      if (lastStatement.kind === "ExpressionStatement") {
        const exprStmt = lastStatement as AST.ExpressionStatement
        return this.generateConstraintsForExpression(
          exprStmt.expression,
          blockEnv
        )
      }
    }

    // If no return expression and last statement is not an expression, return Unit
    return new AST.PrimitiveType("Unit", block.line, block.column)
  }

  private generateConstraintsForFunctorMap(
    functorMap: AST.FunctorMap,
    env: Map<string, AST.Type>
  ): AST.Type {
    // <$> operator: f <$> m
    // Type signature: (a -> b) -> f a -> f b
    const funcType = this.generateConstraintsForExpression(functorMap.left, env)
    const containerType = this.generateConstraintsForExpression(
      functorMap.right,
      env
    )

    const inputType = this.freshTypeVariable(functorMap.line, functorMap.column)
    const outputType = this.freshTypeVariable(
      functorMap.line,
      functorMap.column
    )

    // Function must be of type (a -> b)
    const expectedFuncType = new AST.FunctionType(
      inputType,
      outputType,
      functorMap.line,
      functorMap.column
    )

    this.addConstraint(
      new TypeConstraint(
        funcType,
        expectedFuncType,
        functorMap.line,
        functorMap.column,
        `FunctorMap function type`
      )
    )

    // Handle case where containerType might be TypeVariable
    if (containerType.kind === "GenericType") {
      const gt = containerType as AST.GenericType

      if (gt.name === "Maybe" && gt.typeArguments.length === 1) {
        // Maybe<a> case
        this.addConstraint(
          new TypeConstraint(
            gt.typeArguments[0],
            inputType,
            functorMap.line,
            functorMap.column,
            `FunctorMap Maybe container input type`
          )
        )

        return new AST.GenericType(
          "Maybe",
          [outputType],
          functorMap.line,
          functorMap.column
        )
      } else if (gt.name === "Either" && gt.typeArguments.length === 2) {
        // Either<e, a> case - function operates on the 'a' type (right side)
        const errorType = gt.typeArguments[0] // Left type stays the same
        this.addConstraint(
          new TypeConstraint(
            gt.typeArguments[1], // Right type
            inputType,
            functorMap.line,
            functorMap.column,
            `FunctorMap Either container input type`
          )
        )

        return new AST.GenericType(
          "Either",
          [errorType, outputType], // Keep error type, map value type
          functorMap.line,
          functorMap.column
        )
      } else if (gt.name === "List" && gt.typeArguments.length === 1) {
        // List<a> case
        this.addConstraint(
          new TypeConstraint(
            gt.typeArguments[0],
            inputType,
            functorMap.line,
            functorMap.column,
            `FunctorMap List container input type`
          )
        )

        return new AST.GenericType(
          "List",
          [outputType],
          functorMap.line,
          functorMap.column
        )
      } else if (gt.name === "Task" && gt.typeArguments.length === 1) {
        // Task<a> case
        this.addConstraint(
          new TypeConstraint(
            gt.typeArguments[0],
            inputType,
            functorMap.line,
            functorMap.column,
            `FunctorMap Task container input type`
          )
        )

        return new AST.GenericType(
          "Task",
          [outputType],
          functorMap.line,
          functorMap.column
        )
      } else if (gt.typeArguments.length > 0) {
        // Generic functor case
        this.addConstraint(
          new TypeConstraint(
            gt.typeArguments[gt.typeArguments.length - 1], // Last type argument
            inputType,
            functorMap.line,
            functorMap.column,
            `FunctorMap container input type`
          )
        )

        // Replace last type argument with output type
        const newArgs = [...gt.typeArguments]
        newArgs[newArgs.length - 1] = outputType

        return new AST.GenericType(
          gt.name,
          newArgs,
          functorMap.line,
          functorMap.column
        )
      }
    } else if (containerType.kind === "TypeVariable") {
      // Container is a type variable - we need to create constraints that will be resolved later
      const resultType = this.freshTypeVariable(
        functorMap.line,
        functorMap.column
      )

      // Create a special constraint for functor mapping with type variables
      // This will be resolved during constraint solving when the containerType is unified
      const functorMapConstraint = new FunctorMapConstraint(
        containerType,
        inputType,
        outputType,
        resultType,
        functorMap.line,
        functorMap.column,
        "FunctorMap with type variable container"
      )

      this.constraints.push(functorMapConstraint)
      return resultType
    }

    // Fallback: create a type variable instead of forcing Functor type
    const resultType = this.freshTypeVariable(
      functorMap.line,
      functorMap.column
    )
    return resultType
  }

  private generateConstraintsForApplicativeApply(
    applicativeApply: AST.ApplicativeApply,
    env: Map<string, AST.Type>
  ): AST.Type {
    // <*> operator: f (a -> b) <*> f a
    // Type signature: f (a -> b) -> f a -> f b
    const funcContainerType = this.generateConstraintsForExpression(
      applicativeApply.left,
      env
    )
    const valueContainerType = this.generateConstraintsForExpression(
      applicativeApply.right,
      env
    )

    const inputType = this.freshTypeVariable(
      applicativeApply.line,
      applicativeApply.column
    )
    const outputType = this.freshTypeVariable(
      applicativeApply.line,
      applicativeApply.column
    )

    // Function type inside the container
    const funcType = new AST.FunctionType(
      inputType,
      outputType,
      applicativeApply.line,
      applicativeApply.column
    )

    // Handle case where funcContainerType is GenericType but valueContainerType might be TypeVariable
    if (funcContainerType.kind === "GenericType") {
      const funcGt = funcContainerType as AST.GenericType

      // If valueContainerType is also GenericType, ensure they match
      if (valueContainerType.kind === "GenericType") {
        const valueGt = valueContainerType as AST.GenericType

        // Ensure both containers are of the same type
        if (funcGt.name !== valueGt.name) {
          // Type mismatch - add constraint to unify them
          this.addConstraint(
            new TypeConstraint(
              funcContainerType,
              valueContainerType,
              applicativeApply.line,
              applicativeApply.column,
              `ApplicativeApply container type mismatch`
            )
          )
        }
      } else {
        // valueContainerType is TypeVariable - constrain it to match funcContainerType structure
        const expectedValueType = new AST.GenericType(
          funcGt.name,
          [inputType],
          applicativeApply.line,
          applicativeApply.column
        )

        this.addConstraint(
          new TypeConstraint(
            valueContainerType,
            expectedValueType,
            applicativeApply.line,
            applicativeApply.column,
            `ApplicativeApply value container type constraint`
          )
        )
      }

      // Handle specific applicative types based on funcGt
      if (funcGt.name === "Maybe" && funcGt.typeArguments.length === 1) {
        // Maybe case: Maybe<(a -> b)> <*> Maybe<a> -> Maybe<b>
        this.addConstraint(
          new TypeConstraint(
            funcGt.typeArguments[0],
            funcType,
            applicativeApply.line,
            applicativeApply.column,
            `ApplicativeApply Maybe function container type`
          )
        )

        return new AST.GenericType(
          "Maybe",
          [outputType],
          applicativeApply.line,
          applicativeApply.column
        )
      } else if (
        funcGt.name === "Either" &&
        funcGt.typeArguments.length === 2
      ) {
        // Either case: Either<e, (a -> b)> <*> Either<e, a> -> Either<e, b>
        const errorType = funcGt.typeArguments[0]

        this.addConstraint(
          new TypeConstraint(
            funcGt.typeArguments[1],
            funcType,
            applicativeApply.line,
            applicativeApply.column,
            `ApplicativeApply Either function container type`
          )
        )

        return new AST.GenericType(
          "Either",
          [errorType, outputType],
          applicativeApply.line,
          applicativeApply.column
        )
      } else if (funcGt.name === "List" && funcGt.typeArguments.length === 1) {
        // List case: List<(a -> b)> <*> List<a> -> List<b>
        this.addConstraint(
          new TypeConstraint(
            funcGt.typeArguments[0],
            funcType,
            applicativeApply.line,
            applicativeApply.column,
            `ApplicativeApply List function container type`
          )
        )

        return new AST.GenericType(
          "List",
          [outputType],
          applicativeApply.line,
          applicativeApply.column
        )
      } else if (funcGt.name === "Task" && funcGt.typeArguments.length === 1) {
        // Task case: Task<(a -> b)> <*> Task<a> -> Task<b>
        this.addConstraint(
          new TypeConstraint(
            funcGt.typeArguments[0],
            funcType,
            applicativeApply.line,
            applicativeApply.column,
            `ApplicativeApply Task function container type`
          )
        )

        return new AST.GenericType(
          "Task",
          [outputType],
          applicativeApply.line,
          applicativeApply.column
        )
      } else {
        // Generic case for other types
        const funcArgIndex = funcGt.typeArguments.length - 1

        this.addConstraint(
          new TypeConstraint(
            funcGt.typeArguments[funcArgIndex],
            funcType,
            applicativeApply.line,
            applicativeApply.column,
            `ApplicativeApply generic function container type`
          )
        )

        // Preserve other type arguments
        const newArgs = [...funcGt.typeArguments]
        newArgs[funcArgIndex] = outputType

        return new AST.GenericType(
          funcGt.name,
          newArgs,
          applicativeApply.line,
          applicativeApply.column
        )
      }
    }

    // Handle Array type (JavaScript arrays)
    if (
      funcContainerType.kind === "PrimitiveType" &&
      valueContainerType.kind === "PrimitiveType" &&
      (funcContainerType as AST.PrimitiveType).name === "Array" &&
      (valueContainerType as AST.PrimitiveType).name === "Array"
    ) {
      // Array case: Array <*> Array -> Array
      return new AST.PrimitiveType(
        "Array",
        applicativeApply.line,
        applicativeApply.column
      )
    }

    // Handle case where funcContainerType is TypeVariable
    if (funcContainerType.kind === "TypeVariable") {
      // Create a fresh type variable for the result
      const resultType = this.freshTypeVariable(
        applicativeApply.line,
        applicativeApply.column
      )

      // Create a special constraint for applicative apply with type variables
      // This will be resolved during constraint solving when the container types are unified
      const applicativeApplyConstraint = new ApplicativeApplyConstraint(
        funcContainerType,
        valueContainerType,
        inputType,
        outputType,
        resultType,
        applicativeApply.line,
        applicativeApply.column,
        "ApplicativeApply with type variable containers"
      )

      this.constraints.push(applicativeApplyConstraint)
      return resultType
    }

    // Handle case where both are TypeVariables
    if (
      funcContainerType.kind === "TypeVariable" &&
      valueContainerType.kind === "TypeVariable"
    ) {
      // Create a fresh type variable for the result
      const resultType = this.freshTypeVariable(
        applicativeApply.line,
        applicativeApply.column
      )

      // Add constraints that will be resolved later
      // The funcContainer should be of form f<(a -> b)>
      // The valueContainer should be of form f<a>
      // The result should be of form f<b>

      // For now, return the result type variable
      return resultType
    }

    // Fallback - should rarely reach here if type inference is working correctly
    return new AST.GenericType(
      "Applicative",
      [outputType],
      applicativeApply.line,
      applicativeApply.column
    )
  }

  private generateConstraintsForMonadBind(
    monadBind: AST.MonadBind,
    env: Map<string, AST.Type>
  ): AST.Type {
    // >>= operator: m a >>= (a -> m b)
    // Type signature: m a -> (a -> m b) -> m b
    const monadType = this.generateConstraintsForExpression(monadBind.left, env)
    const funcType = this.generateConstraintsForExpression(monadBind.right, env)

    const inputType = this.freshTypeVariable(monadBind.line, monadBind.column)
    const outputType = this.freshTypeVariable(monadBind.line, monadBind.column)

    // Create a generic monad variable that will be constrained later
    const monadVar = this.freshTypeVariable(monadBind.line, monadBind.column)

    // Constrain the left side to be a monad of inputType
    this.addConstraint(
      new TypeConstraint(
        monadType,
        monadVar,
        monadBind.line,
        monadBind.column,
        `MonadBind left side monad type`
      )
    )

    // The function should take inputType and return a monad of outputType
    const expectedOutputMonad = this.freshTypeVariable(
      monadBind.line,
      monadBind.column
    )
    const expectedFuncType = new AST.FunctionType(
      inputType,
      expectedOutputMonad,
      monadBind.line,
      monadBind.column
    )

    this.addConstraint(
      new TypeConstraint(
        funcType,
        expectedFuncType,
        monadBind.line,
        monadBind.column,
        `MonadBind function type`
      )
    )

    // The result should have the same monad structure as the input but with outputType
    const resultType = this.freshTypeVariable(monadBind.line, monadBind.column)

    // Add constraint that the output monad and result should be the same
    this.addConstraint(
      new TypeConstraint(
        expectedOutputMonad,
        resultType,
        monadBind.line,
        monadBind.column,
        `MonadBind result type`
      )
    )

    // Add specific constraints for known monad types
    if (monadType.kind === "GenericType") {
      const monadGt = monadType as AST.GenericType

      if (monadGt.name === "Maybe" && monadGt.typeArguments.length === 1) {
        // Maybe case: Maybe<a> >>= (a -> Maybe<b>) -> Maybe<b>
        this.addConstraint(
          new TypeConstraint(
            monadGt.typeArguments[0],
            inputType,
            monadBind.line,
            monadBind.column,
            `MonadBind Maybe input type`
          )
        )

        const outputMonadType = new AST.GenericType(
          "Maybe",
          [outputType],
          monadBind.line,
          monadBind.column
        )

        this.addConstraint(
          new TypeConstraint(
            expectedOutputMonad,
            outputMonadType,
            monadBind.line,
            monadBind.column,
            `MonadBind Maybe output type`
          )
        )

        return new AST.GenericType(
          "Maybe",
          [outputType],
          monadBind.line,
          monadBind.column
        )
      } else if (
        monadGt.name === "Either" &&
        monadGt.typeArguments.length === 2
      ) {
        // Either case: Either<e, a> >>= (a -> Either<e, b>) -> Either<e, b>
        const errorType = monadGt.typeArguments[0]
        const valueType = monadGt.typeArguments[1]

        this.addConstraint(
          new TypeConstraint(
            valueType,
            inputType,
            monadBind.line,
            monadBind.column,
            `MonadBind Either input type`
          )
        )

        const outputMonadType = new AST.GenericType(
          "Either",
          [errorType, outputType],
          monadBind.line,
          monadBind.column
        )

        this.addConstraint(
          new TypeConstraint(
            expectedOutputMonad,
            outputMonadType,
            monadBind.line,
            monadBind.column,
            `MonadBind Either output type`
          )
        )

        return new AST.GenericType(
          "Either",
          [errorType, outputType],
          monadBind.line,
          monadBind.column
        )
      } else if (
        monadGt.name === "List" &&
        monadGt.typeArguments.length === 1
      ) {
        // List case: List<a> >>= (a -> List<b>) -> List<b>
        this.addConstraint(
          new TypeConstraint(
            monadGt.typeArguments[0],
            inputType,
            monadBind.line,
            monadBind.column,
            `MonadBind List input type`
          )
        )

        const outputMonadType = new AST.GenericType(
          "List",
          [outputType],
          monadBind.line,
          monadBind.column
        )

        this.addConstraint(
          new TypeConstraint(
            expectedOutputMonad,
            outputMonadType,
            monadBind.line,
            monadBind.column,
            `MonadBind List output type`
          )
        )

        return new AST.GenericType(
          "List",
          [outputType],
          monadBind.line,
          monadBind.column
        )
      } else if (
        monadGt.name === "Task" &&
        monadGt.typeArguments.length === 1
      ) {
        // Task case: Task<a> >>= (a -> Task<b>) -> Task<b>
        this.addConstraint(
          new TypeConstraint(
            monadGt.typeArguments[0],
            inputType,
            monadBind.line,
            monadBind.column,
            `MonadBind Task input type`
          )
        )

        const outputMonadType = new AST.GenericType(
          "Task",
          [outputType],
          monadBind.line,
          monadBind.column
        )

        this.addConstraint(
          new TypeConstraint(
            expectedOutputMonad,
            outputMonadType,
            monadBind.line,
            monadBind.column,
            `MonadBind Task output type`
          )
        )

        return new AST.GenericType(
          "Task",
          [outputType],
          monadBind.line,
          monadBind.column
        )
      }
    }

    // Handle Array type (JavaScript arrays)
    if (
      monadType.kind === "PrimitiveType" &&
      (monadType as AST.PrimitiveType).name === "Array"
    ) {
      // Array case: Array >>= (any -> Array) -> Array
      return new AST.PrimitiveType("Array", monadBind.line, monadBind.column)
    }

    // For unknown types, let constraint resolution figure it out
    return resultType
  }

  private generateConstraintsForFunctionApplicationOperator(
    funcApp: AST.FunctionApplicationOperator,
    env: Map<string, AST.Type>
  ): AST.Type {
    // $ operator: f $ x = f(x)
    // This is the same as function application but with infix syntax
    const funcType = this.generateConstraintsForExpression(funcApp.left, env)
    const argType = this.generateConstraintsForExpression(funcApp.right, env)

    const resultType = this.freshTypeVariable(funcApp.line, funcApp.column)

    // The function should be of type argType -> resultType
    const expectedFuncType = new AST.FunctionType(
      argType,
      resultType,
      funcApp.line,
      funcApp.column
    )

    this.addConstraint(
      new TypeConstraint(
        funcType,
        expectedFuncType,
        funcApp.line,
        funcApp.column,
        `Function application operator $`
      )
    )

    return resultType
  }

  private generateConstraintsForConstructorExpression(
    ctor: AST.ConstructorExpression,
    env: Map<string, AST.Type>
  ): AST.Type {
    const constructorName = ctor.constructorName

    switch (constructorName) {
      case "Just":
        if (ctor.arguments && ctor.arguments.length > 0) {
          const argType = this.generateConstraintsForExpression(
            ctor.arguments[0],
            env
          )
          return new AST.GenericType("Maybe", [argType], ctor.line, ctor.column)
        } else if (!ctor.arguments || ctor.arguments.length === 0) {
          // Just without arguments - treat as a curried function
          // Just : 'a -> Maybe<'a>
          const elemType = new PolymorphicTypeVariable(
            "a",
            ctor.line,
            ctor.column
          )
          const maybeType = new AST.GenericType(
            "Maybe",
            [elemType],
            ctor.line,
            ctor.column
          )
          return new AST.FunctionType(
            elemType,
            maybeType,
            ctor.line,
            ctor.column
          )
        }
        // Just with wrong number of arguments - should be error
        this.errors.push(
          new TypeInferenceError(
            "Just constructor requires exactly one argument",
            ctor.line,
            ctor.column
          )
        )
        return new AST.GenericType(
          "Maybe",
          [this.freshTypeVariable(ctor.line, ctor.column)],
          ctor.line,
          ctor.column
        )

      case "Nothing":
        // Nothing doesn't take arguments
        if (ctor.arguments && ctor.arguments.length > 0) {
          this.errors.push(
            new TypeInferenceError(
              "Nothing constructor does not take any arguments",
              ctor.line,
              ctor.column
            )
          )
        }
        // Nothing is polymorphic: Maybe<'a>
        return new AST.GenericType(
          "Maybe",
          [new PolymorphicTypeVariable("a", ctor.line, ctor.column)],
          ctor.line,
          ctor.column
        )

      case "Right":
        if (ctor.arguments && ctor.arguments.length > 0) {
          const argType = this.generateConstraintsForExpression(
            ctor.arguments[0],
            env
          )
          // Right is polymorphic in its left type: Either<'a, argType>
          return new AST.GenericType(
            "Either",
            [new PolymorphicTypeVariable("a", ctor.line, ctor.column), argType],
            ctor.line,
            ctor.column
          )
        } else if (!ctor.arguments || ctor.arguments.length === 0) {
          // Right without arguments - treat as a curried function
          // Right : 'b -> Either<'a, 'b>
          const leftType = new PolymorphicTypeVariable(
            "a",
            ctor.line,
            ctor.column
          )
          const rightType = new PolymorphicTypeVariable(
            "b",
            ctor.line,
            ctor.column
          )
          const eitherType = new AST.GenericType(
            "Either",
            [leftType, rightType],
            ctor.line,
            ctor.column
          )
          return new AST.FunctionType(
            rightType,
            eitherType,
            ctor.line,
            ctor.column
          )
        }
        this.errors.push(
          new TypeInferenceError(
            "Right constructor requires exactly one argument",
            ctor.line,
            ctor.column
          )
        )
        return new AST.GenericType(
          "Either",
          [
            new PolymorphicTypeVariable("a", ctor.line, ctor.column),
            new PolymorphicTypeVariable("b", ctor.line, ctor.column),
          ],
          ctor.line,
          ctor.column
        )

      case "Left":
        if (ctor.arguments && ctor.arguments.length > 0) {
          const argType = this.generateConstraintsForExpression(
            ctor.arguments[0],
            env
          )
          // Left is polymorphic in its right type: Either<argType, 'b>
          return new AST.GenericType(
            "Either",
            [argType, new PolymorphicTypeVariable("b", ctor.line, ctor.column)],
            ctor.line,
            ctor.column
          )
        } else if (!ctor.arguments || ctor.arguments.length === 0) {
          // Left without arguments - treat as a curried function
          // Left : 'a -> Either<'a, 'b>
          const leftType = new PolymorphicTypeVariable(
            "a",
            ctor.line,
            ctor.column
          )
          const rightType = new PolymorphicTypeVariable(
            "b",
            ctor.line,
            ctor.column
          )
          const eitherType = new AST.GenericType(
            "Either",
            [leftType, rightType],
            ctor.line,
            ctor.column
          )
          return new AST.FunctionType(
            leftType,
            eitherType,
            ctor.line,
            ctor.column
          )
        }
        this.errors.push(
          new TypeInferenceError(
            "Left constructor requires exactly one argument",
            ctor.line,
            ctor.column
          )
        )
        return new AST.GenericType(
          "Either",
          [
            new PolymorphicTypeVariable("a", ctor.line, ctor.column),
            new PolymorphicTypeVariable("b", ctor.line, ctor.column),
          ],
          ctor.line,
          ctor.column
        )

      case "Task":
        if (ctor.arguments && ctor.arguments.length > 0) {
          const argType = this.generateConstraintsForExpression(
            ctor.arguments[0],
            env
          )
          // Task expects (() -> Promise<T>) -> Task<T>
          // So the argument should be a computation of type () -> Promise<T>
          const typeVar = new PolymorphicTypeVariable(
            "a",
            ctor.line,
            ctor.column
          )
          const promiseType = new AST.GenericType(
            "Promise",
            [typeVar],
            ctor.line,
            ctor.column
          )
          const expectedArgType = new AST.FunctionType(
            new AST.PrimitiveType("Unit", ctor.line, ctor.column),
            promiseType,
            ctor.line,
            ctor.column
          )

          // Add constraint that argument must be () -> Promise<T>
          this.addConstraint(
            new TypeConstraint(
              argType,
              expectedArgType,
              ctor.line,
              ctor.column,
              "Task constructor argument constraint"
            )
          )

          return new AST.GenericType("Task", [typeVar], ctor.line, ctor.column)
        } else if (!ctor.arguments || ctor.arguments.length === 0) {
          // Task without arguments - treat as a curried function
          // Task : (() -> Promise<'a>) -> Task<'a>
          const typeVar = new PolymorphicTypeVariable(
            "a",
            ctor.line,
            ctor.column
          )
          const promiseType = new AST.GenericType(
            "Promise",
            [typeVar],
            ctor.line,
            ctor.column
          )
          const computationType = new AST.FunctionType(
            new AST.PrimitiveType("Unit", ctor.line, ctor.column),
            promiseType,
            ctor.line,
            ctor.column
          )
          const taskType = new AST.GenericType(
            "Task",
            [typeVar],
            ctor.line,
            ctor.column
          )
          return new AST.FunctionType(
            computationType,
            taskType,
            ctor.line,
            ctor.column
          )
        }
        this.errors.push(
          new TypeInferenceError(
            "Task constructor requires exactly one argument",
            ctor.line,
            ctor.column
          )
        )
        return new AST.GenericType(
          "Task",
          [new PolymorphicTypeVariable("a", ctor.line, ctor.column)],
          ctor.line,
          ctor.column
        )

      case "Empty":
        // Empty is polymorphic: List<'a>
        return new AST.GenericType(
          "List",
          [new PolymorphicTypeVariable("a", ctor.line, ctor.column)],
          ctor.line,
          ctor.column
        )

      case "Cons":
        if (ctor.arguments && ctor.arguments.length === 2) {
          const headType = this.generateConstraintsForExpression(
            ctor.arguments[0],
            env
          )
          const tailType = this.generateConstraintsForExpression(
            ctor.arguments[1],
            env
          )

          // Cons head tail should have type List<headType>
          const expectedTailType = new AST.GenericType(
            "List",
            [headType],
            ctor.line,
            ctor.column
          )

          // Add constraint that tail must be List<headType>
          this.addConstraint(
            new TypeConstraint(
              tailType,
              expectedTailType,
              ctor.line,
              ctor.column,
              "Cons tail type"
            )
          )

          return expectedTailType
        } else if (!ctor.arguments || ctor.arguments.length === 0) {
          // Cons without arguments - treat as a curried function
          // Cons : 'a -> List<'a> -> List<'a>
          const elemType = new PolymorphicTypeVariable(
            "a",
            ctor.line,
            ctor.column
          )
          const listType = new AST.GenericType(
            "List",
            [elemType],
            ctor.line,
            ctor.column
          )
          return new AST.FunctionType(
            elemType,
            new AST.FunctionType(listType, listType, ctor.line, ctor.column),
            ctor.line,
            ctor.column
          )
        }
        this.errors.push(
          new TypeInferenceError(
            "Cons constructor requires exactly two arguments (head and tail)",
            ctor.line,
            ctor.column
          )
        )
        return new AST.GenericType(
          "List",
          [new PolymorphicTypeVariable("a", ctor.line, ctor.column)],
          ctor.line,
          ctor.column
        )

      default: {
        // Check if this is an ADT constructor from the environment
        const constructorType = env.get(constructorName)
        if (constructorType) {
          // This is a known ADT constructor
          return this.applyConstructor(constructorType, ctor, env)
        }

        this.errors.push(
          new TypeInferenceError(
            `Unknown constructor: ${constructorName}`,
            ctor.line,
            ctor.column
          )
        )
        return this.freshTypeVariable(ctor.line, ctor.column)
      }
    }
  }

  private applyConstructor(
    constructorType: AST.Type,
    ctor: AST.ConstructorExpression,
    env: Map<string, AST.Type>
  ): AST.Type {
    // Extract parameter types and result type from constructor function type
    let currentType = constructorType
    const expectedParamTypes: AST.Type[] = []

    // Traverse function type to get parameter types
    while (currentType instanceof AST.FunctionType) {
      expectedParamTypes.push(currentType.paramType)
      currentType = currentType.returnType
    }

    // The final type should be the ADT type
    const resultType = currentType

    // Check argument count
    if (ctor.arguments.length !== expectedParamTypes.length) {
      this.errors.push(
        new TypeInferenceError(
          `Constructor ${ctor.constructorName} expects ${expectedParamTypes.length} arguments, but got ${ctor.arguments.length}`,
          ctor.line,
          ctor.column
        )
      )
      return resultType
    }

    // Type check each argument
    for (let i = 0; i < ctor.arguments.length; i++) {
      const argType = this.generateConstraintsForExpression(
        ctor.arguments[i],
        env
      )

      // Add constraint that argument type matches expected parameter type
      this.addConstraint(
        new TypeConstraint(
          argType,
          expectedParamTypes[i],
          ctor.arguments[i].line,
          ctor.arguments[i].column,
          `Constructor ${ctor.constructorName} argument ${i + 1}`
        )
      )
    }

    return resultType
  }

  private generateConstraintsForSignalExpression(
    signal: AST.SignalExpression,
    env: Map<string, AST.Type>
  ): AST.Type {
    // Signal<T>の型を推論
    const valueType = this.generateConstraintsForExpression(
      signal.initialValue,
      env
    )
    return new AST.GenericType(
      "Signal",
      [valueType],
      signal.line,
      signal.column
    )
  }

  private generateConstraintsForAssignmentExpression(
    assignment: AST.AssignmentExpression,
    env: Map<string, AST.Type>
  ): AST.Type {
    // target（代入先）の型を推論
    const targetType = this.generateConstraintsForExpression(
      assignment.target,
      env
    )
    // value（代入する値）の型を推論
    const valueType = this.generateConstraintsForExpression(
      assignment.value,
      env
    )

    // targetがSignal<T>型かチェック
    if (targetType.kind === "GenericType") {
      const genType = targetType as AST.GenericType
      if (genType.name === "Signal" && genType.typeArguments.length === 1) {
        const signalElementType = genType.typeArguments[0]

        // valueが T または T -> T のどちらかをチェック
        // 関数型かどうかを構文的に判定
        if (assignment.value.kind === "LambdaExpression") {
          // Lambda式の場合は関数型として扱う
          const functionType = new AST.FunctionType(
            signalElementType,
            signalElementType,
            assignment.value.line,
            assignment.value.column
          )
          this.addConstraint(
            new TypeConstraint(
              valueType,
              functionType,
              assignment.value.line,
              assignment.value.column,
              "Signal assignment function type"
            )
          )
        } else {
          // その他の場合は直接値として扱う
          this.addConstraint(
            new TypeConstraint(
              valueType,
              signalElementType,
              assignment.value.line,
              assignment.value.column,
              "Signal assignment value type"
            )
          )
        }

        // Signal代入は代入先のSignal型を返す
        return targetType
      }
    }

    this.errors.push(
      new TypeInferenceError(
        "Assignment target must be a Signal type",
        assignment.target.line,
        assignment.target.column
      )
    )
    return this.freshTypeVariable(assignment.line, assignment.column)
  }

  private generateConstraintsForLambdaExpression(
    lambda: AST.LambdaExpression,
    env: Map<string, AST.Type>
  ): AST.Type {
    // Create a new environment for the lambda body
    const lambdaEnv = new Map(env)

    // Create fresh type variables for parameters with placeholder types
    const parameterTypes: AST.Type[] = []

    for (const param of lambda.parameters) {
      let paramType = param.type

      // If parameter type is placeholder "_", create fresh type variable
      if (
        paramType.kind === "PrimitiveType" &&
        (paramType as AST.PrimitiveType).name === "_"
      ) {
        paramType = this.freshTypeVariable(param.line, param.column)
      }

      parameterTypes.push(paramType)
      lambdaEnv.set(param.name, paramType)
    }

    // Infer the type of the lambda body
    const bodyType = this.generateConstraintsForExpression(
      lambda.body,
      lambdaEnv
    )

    // Build the function type from right to left (currying)
    let resultType: AST.Type = bodyType
    for (let i = lambda.parameters.length - 1; i >= 0; i--) {
      resultType = new AST.FunctionType(
        parameterTypes[i],
        resultType,
        lambda.line,
        lambda.column
      )
    }

    return resultType
  }

  // 制約解決（単一化アルゴリズム）
  private solveConstraints(): TypeSubstitution {
    let substitution = new TypeSubstitution()

    for (const constraint of this.constraints) {
      try {
        if (constraint instanceof ArrayAccessConstraint) {
          // ArrayAccessConstraintの特別な処理
          const constraintSub = this.solveArrayAccessConstraint(
            constraint,
            substitution
          )
          substitution = substitution.compose(constraintSub)
        } else if (constraint instanceof FunctorMapConstraint) {
          // FunctorMapConstraintの特別な処理
          const constraintSub = this.solveFunctorMapConstraint(
            constraint,
            substitution
          )
          substitution = substitution.compose(constraintSub)
        } else if (constraint instanceof ApplicativeApplyConstraint) {
          // ApplicativeApplyConstraintの特別な処理
          const constraintSub = this.solveApplicativeApplyConstraint(
            constraint,
            substitution
          )
          substitution = substitution.compose(constraintSub)
        } else {
          // 通常のTypeConstraint処理
          const constraintSub = this.unify(
            substitution.apply(constraint.type1),
            substitution.apply(constraint.type2)
          )
          substitution = substitution.compose(constraintSub)
        }
      } catch (error) {
        this.errors.push(
          new TypeInferenceError(
            `Cannot unify types: ${error}`,
            constraint.line,
            constraint.column,
            constraint.context
          )
        )
      }
    }

    // 部分型制約を解決（一旧無効化）
    // for (const subtypeConstraint of this.subtypeConstraints) {
    //   try {
    //     const subType = substitution.apply(subtypeConstraint.subType)
    //     const superType = substitution.apply(subtypeConstraint.superType)

    //     if (!this.isSubtype(subType, superType)) {
    //       this.errors.push(
    //         new TypeInferenceError(
    //           `Subtype constraint violated: ${typeToStringUtil(subType)} is not a subtype of ${typeToStringUtil(superType)}`,
    //           subtypeConstraint.line,
    //           subtypeConstraint.column,
    //           subtypeConstraint.context
    //         )
    //       )
    //     }
    //   } catch (error) {
    //     this.errors.push(
    //       new TypeInferenceError(
    //         `Error checking subtype constraint: ${error}`,
    //         subtypeConstraint.line,
    //         subtypeConstraint.column,
    //         subtypeConstraint.context
    //       )
    //     )
    //   }
    // }

    return substitution
  }

  // 部分的制約解決：指定した制約のみを解決
  private solveConstraintsPartial(
    constraintsToSolve: (
      | TypeConstraint
      | ArrayAccessConstraint
      | FunctorMapConstraint
      | ApplicativeApplyConstraint
    )[]
  ): TypeSubstitution {
    let substitution = new TypeSubstitution()

    for (const constraint of constraintsToSolve) {
      try {
        if (constraint instanceof ArrayAccessConstraint) {
          // ArrayAccessConstraintの特別な処理
          const constraintSub = this.solveArrayAccessConstraint(
            constraint,
            substitution
          )
          substitution = substitution.compose(constraintSub)
        } else if (constraint instanceof FunctorMapConstraint) {
          // FunctorMapConstraintの特別な処理
          const constraintSub = this.solveFunctorMapConstraint(
            constraint,
            substitution
          )
          substitution = substitution.compose(constraintSub)
        } else if (constraint instanceof ApplicativeApplyConstraint) {
          // ApplicativeApplyConstraintの特別な処理
          const constraintSub = this.solveApplicativeApplyConstraint(
            constraint,
            substitution
          )
          substitution = substitution.compose(constraintSub)
        } else {
          // 通常のTypeConstraint処理
          const constraintSub = this.unify(
            substitution.apply(constraint.type1),
            substitution.apply(constraint.type2)
          )
          substitution = substitution.compose(constraintSub)
        }
      } catch (error) {
        // 部分解決では、エラーが起きても他の制約解決を続行
      }
    }

    return substitution
  }

  // ArrayAccess制約の特別な解決
  private solveArrayAccessConstraint(
    constraint: ArrayAccessConstraint,
    currentSubstitution: TypeSubstitution
  ): TypeSubstitution {
    const arrayType = currentSubstitution.apply(constraint.arrayType)
    const resultType = constraint.resultType

    // Array<T>の場合
    if (arrayType.kind === "GenericType") {
      const gt = arrayType as AST.GenericType
      if (gt.name === "Array" && gt.typeArguments.length === 1) {
        // Array<T>[index] -> Maybe<T>
        const maybeType = new AST.GenericType(
          "Maybe",
          [gt.typeArguments[0]],
          constraint.line,
          constraint.column
        )
        return this.unify(resultType, maybeType)
      }
    }

    // Tuple型の場合
    if (arrayType.kind === "TupleType") {
      const tt = arrayType as AST.TupleType
      if (tt.elementTypes.length > 0) {
        // タプルアクセスの場合、TypeScriptの挙動に合わせて
        // すべての要素型のunion型として扱う
        // ただし、型安全性のため、結果型を任意の型変数とする
        const unionType = this.freshTypeVariable(
          constraint.line,
          constraint.column
        )
        return this.unify(resultType, unionType)
      }
    }

    // 型変数の場合、Array<T>またはTuple型として推論
    if (arrayType.kind === "TypeVariable") {
      const _tv = arrayType as TypeVariable
      const elementType = this.freshTypeVariable(
        constraint.line,
        constraint.column
      )

      // Array<T>として推論
      const arrayGenericType = new AST.GenericType(
        "Array",
        [elementType],
        constraint.line,
        constraint.column
      )

      // 結果はMaybe<T>
      const maybeType = new AST.GenericType(
        "Maybe",
        [elementType],
        constraint.line,
        constraint.column
      )

      const sub1 = this.unify(arrayType, arrayGenericType)
      const sub2 = this.unify(resultType, maybeType)
      return sub1.compose(sub2)
    }

    throw new Error(
      `Array access requires Array<T> or Tuple type, got ${typeToStringUtil(arrayType)}`
    )
  }

  // FunctorMap制約の特別な解決
  private solveFunctorMapConstraint(
    constraint: FunctorMapConstraint,
    currentSubstitution: TypeSubstitution
  ): TypeSubstitution {
    const containerType = currentSubstitution.apply(constraint.containerType)
    const inputType = currentSubstitution.apply(constraint.inputType)
    const outputType = currentSubstitution.apply(constraint.outputType)
    const resultType = constraint.resultType

    // 解決されたcontainerTypeに基づいてFunctorMapを適用
    if (containerType.kind === "GenericType") {
      const gt = containerType as AST.GenericType

      if (gt.name === "Maybe" && gt.typeArguments.length === 1) {
        // Maybe<a> case
        const sub1 = this.unify(gt.typeArguments[0], inputType)
        const maybeResultType = new AST.GenericType(
          "Maybe",
          [outputType],
          constraint.line,
          constraint.column
        )
        const sub2 = this.unify(resultType, maybeResultType)
        return sub1.compose(sub2)
      } else if (gt.name === "Either" && gt.typeArguments.length === 2) {
        // Either<e, a> case
        const errorType = gt.typeArguments[0]
        const sub1 = this.unify(gt.typeArguments[1], inputType)
        const eitherResultType = new AST.GenericType(
          "Either",
          [errorType, outputType],
          constraint.line,
          constraint.column
        )
        const sub2 = this.unify(resultType, eitherResultType)
        return sub1.compose(sub2)
      } else if (gt.name === "List" && gt.typeArguments.length === 1) {
        // List<a> case
        const sub1 = this.unify(gt.typeArguments[0], inputType)
        const listResultType = new AST.GenericType(
          "List",
          [outputType],
          constraint.line,
          constraint.column
        )
        const sub2 = this.unify(resultType, listResultType)
        return sub1.compose(sub2)
      } else if (gt.name === "Task" && gt.typeArguments.length === 1) {
        // Task<a> case
        const sub1 = this.unify(gt.typeArguments[0], inputType)
        const taskResultType = new AST.GenericType(
          "Task",
          [outputType],
          constraint.line,
          constraint.column
        )
        const sub2 = this.unify(resultType, taskResultType)
        return sub1.compose(sub2)
      } else if (gt.typeArguments.length > 0) {
        // Generic functor case
        const sub1 = this.unify(
          gt.typeArguments[gt.typeArguments.length - 1],
          inputType
        )
        const newArgs = [...gt.typeArguments]
        newArgs[newArgs.length - 1] = outputType
        const genericResultType = new AST.GenericType(
          gt.name,
          newArgs,
          constraint.line,
          constraint.column
        )
        const sub2 = this.unify(resultType, genericResultType)
        return sub1.compose(sub2)
      }
    }

    // まだ解決されていない型変数の場合、制約を残す
    if (containerType.kind === "TypeVariable") {
      // まだ解決されていないので、制約を再度キューに追加する仕組みが必要
      // 今回は簡単にidentity substitutionを返す
      return new TypeSubstitution()
    }

    throw new Error(
      `FunctorMap requires a Functor type (Maybe, Either, List, Task, etc.), got ${typeToStringUtil(containerType)}`
    )
  }

  private solveApplicativeApplyConstraint(
    constraint: ApplicativeApplyConstraint,
    currentSubstitution: TypeSubstitution
  ): TypeSubstitution {
    const funcContainerType = currentSubstitution.apply(
      constraint.funcContainerType
    )
    const valueContainerType = currentSubstitution.apply(
      constraint.valueContainerType
    )
    const inputType = currentSubstitution.apply(constraint.inputType)
    const outputType = currentSubstitution.apply(constraint.outputType)
    const resultType = constraint.resultType

    // 解決されたfuncContainerTypeに基づいてApplicativeApplyを適用
    if (funcContainerType.kind === "GenericType") {
      const funcGt = funcContainerType as AST.GenericType

      // valueContainerTypeも同じコンテナ型に統一する必要がある

      if (funcGt.name === "Maybe" && funcGt.typeArguments.length === 1) {
        // Maybe<(a -> b)> <*> Maybe<a> -> Maybe<b>
        const funcType = new AST.FunctionType(
          inputType,
          outputType,
          constraint.line,
          constraint.column
        )
        const sub1 = this.unify(funcGt.typeArguments[0], funcType)

        // valueContainerTypeをMaybe<inputType>に統一
        const expectedValueType = new AST.GenericType(
          "Maybe",
          [inputType],
          constraint.line,
          constraint.column
        )
        const sub2 = this.unify(valueContainerType, expectedValueType)

        // 結果をMaybe<outputType>に統一
        const maybeResultType = new AST.GenericType(
          "Maybe",
          [outputType],
          constraint.line,
          constraint.column
        )
        const sub3 = this.unify(resultType, maybeResultType)

        return sub1.compose(sub2).compose(sub3)
      } else if (
        funcGt.name === "Either" &&
        funcGt.typeArguments.length === 2
      ) {
        // Either<e, (a -> b)> <*> Either<e, a> -> Either<e, b>
        const errorType = funcGt.typeArguments[0]
        const funcType = new AST.FunctionType(
          inputType,
          outputType,
          constraint.line,
          constraint.column
        )
        const sub1 = this.unify(funcGt.typeArguments[1], funcType)

        // valueContainerTypeをEither<errorType, inputType>に統一
        const expectedValueType = new AST.GenericType(
          "Either",
          [errorType, inputType],
          constraint.line,
          constraint.column
        )
        const sub2 = this.unify(valueContainerType, expectedValueType)

        // 結果をEither<errorType, outputType>に統一
        const eitherResultType = new AST.GenericType(
          "Either",
          [errorType, outputType],
          constraint.line,
          constraint.column
        )
        const sub3 = this.unify(resultType, eitherResultType)

        return sub1.compose(sub2).compose(sub3)
      } else if (funcGt.name === "List" && funcGt.typeArguments.length === 1) {
        // List<(a -> b)> <*> List<a> -> List<b>
        const funcType = new AST.FunctionType(
          inputType,
          outputType,
          constraint.line,
          constraint.column
        )
        const sub1 = this.unify(funcGt.typeArguments[0], funcType)

        // valueContainerTypeをList<inputType>に統一
        const expectedValueType = new AST.GenericType(
          "List",
          [inputType],
          constraint.line,
          constraint.column
        )
        const sub2 = this.unify(valueContainerType, expectedValueType)

        // 結果をList<outputType>に統一
        const listResultType = new AST.GenericType(
          "List",
          [outputType],
          constraint.line,
          constraint.column
        )
        const sub3 = this.unify(resultType, listResultType)

        return sub1.compose(sub2).compose(sub3)
      } else if (funcGt.name === "Task" && funcGt.typeArguments.length === 1) {
        // Task<(a -> b)> <*> Task<a> -> Task<b>
        const funcType = new AST.FunctionType(
          inputType,
          outputType,
          constraint.line,
          constraint.column
        )
        const sub1 = this.unify(funcGt.typeArguments[0], funcType)

        // valueContainerTypeをTask<inputType>に統一
        const expectedValueType = new AST.GenericType(
          "Task",
          [inputType],
          constraint.line,
          constraint.column
        )
        const sub2 = this.unify(valueContainerType, expectedValueType)

        // 結果をTask<outputType>に統一
        const taskResultType = new AST.GenericType(
          "Task",
          [outputType],
          constraint.line,
          constraint.column
        )
        const sub3 = this.unify(resultType, taskResultType)

        return sub1.compose(sub2).compose(sub3)
      } else if (funcGt.typeArguments.length > 0) {
        // Generic applicative case
        const funcArgIndex = funcGt.typeArguments.length - 1
        const funcType = new AST.FunctionType(
          inputType,
          outputType,
          constraint.line,
          constraint.column
        )
        const sub1 = this.unify(funcGt.typeArguments[funcArgIndex], funcType)

        // valueContainerTypeを同じコンテナ型に統一
        const valueArgs = [...funcGt.typeArguments]
        valueArgs[funcArgIndex] = inputType
        const expectedValueType = new AST.GenericType(
          funcGt.name,
          valueArgs,
          constraint.line,
          constraint.column
        )
        const sub2 = this.unify(valueContainerType, expectedValueType)

        // 結果を同じコンテナ型に統一
        const resultArgs = [...funcGt.typeArguments]
        resultArgs[funcArgIndex] = outputType
        const genericResultType = new AST.GenericType(
          funcGt.name,
          resultArgs,
          constraint.line,
          constraint.column
        )
        const sub3 = this.unify(resultType, genericResultType)

        return sub1.compose(sub2).compose(sub3)
      }
    }

    // まだ解決されていない型変数の場合、制約を残す
    if (funcContainerType.kind === "TypeVariable") {
      // まだ解決されていないので、制約を再度キューに追加する仕組みが必要
      // 今回は簡単にidentity substitutionを返す
      return new TypeSubstitution()
    }

    throw new Error(
      `ApplicativeApply requires an Applicative type (Maybe, Either, List, Task, etc.), got ${typeToStringUtil(funcContainerType)}`
    )
  }

  // 単一化アルゴリズム
  private unify(type1: AST.Type, type2: AST.Type): TypeSubstitution {
    // null/undefined チェック
    if (!type1 || !type2) {
      throw new Error(
        `Cannot unify types: one or both types are undefined/null`
      )
    }

    const substitution = new TypeSubstitution()

    // 型エイリアスを解決

    const resolvedType1 = this.resolveTypeAlias(type1)
    const resolvedType2 = this.resolveTypeAlias(type2)

    // 解決後の型のチェック
    if (!resolvedType1 || !resolvedType2) {
      throw new Error(
        `Cannot resolve type aliases: resolved types are undefined/null`
      )
    }

    // 同じ型の場合（解決後の型で比較）
    if (typesEqualUtil(resolvedType1, resolvedType2)) {
      return substitution
    }

    // 型エイリアスの場合、解決前の名前でも比較
    if (type1.kind === "PrimitiveType" && type2.kind === "PrimitiveType") {
      const prim1 = type1 as AST.PrimitiveType
      const prim2 = type2 as AST.PrimitiveType
      if (prim1.name === prim2.name) {
        return substitution
      }
    }

    // 型変数の場合（解決前の元の型で処理）
    if (type1.kind === "TypeVariable") {
      const tv1 = type1 as TypeVariable
      if (occursCheckUtil(tv1.id, resolvedType2)) {
        throw new Error(
          `Infinite type: ${tv1.name} occurs in ${typeToStringUtil(resolvedType2)}`
        )
      }
      substitution.set(tv1.id, resolvedType2)
      return substitution
    }

    if (type2.kind === "TypeVariable") {
      const tv2 = type2 as TypeVariable
      if (occursCheckUtil(tv2.id, resolvedType1)) {
        throw new Error(
          `Infinite type: ${tv2.name} occurs in ${typeToStringUtil(resolvedType1)}`
        )
      }
      substitution.set(tv2.id, resolvedType1)
      return substitution
    }

    // 多相型変数の場合 - これらは統一可能
    if (
      resolvedType1.kind === "PolymorphicTypeVariable" ||
      resolvedType2.kind === "PolymorphicTypeVariable"
    ) {
      // 多相型変数は統一可能（制約なし）
      // 実際の多相型変数の置換は後で実装する
      return substitution
    }

    // プリミティブ型の場合（解決後の型で比較）
    if (
      resolvedType1.kind === "PrimitiveType" &&
      resolvedType2.kind === "PrimitiveType"
    ) {
      const pt1 = resolvedType1 as AST.PrimitiveType
      const pt2 = resolvedType2 as AST.PrimitiveType
      if (pt1.name === pt2.name) {
        return substitution
      }
      throw new Error(
        `Cannot unify ${typeToStringUtil(type1)} with ${typeToStringUtil(type2)}`
      )
    }

    // Function型の場合
    if (type1.kind === "FunctionType" && type2.kind === "FunctionType") {
      const ft1 = type1 as AST.FunctionType
      const ft2 = type2 as AST.FunctionType

      const paramSub = this.unify(ft1.paramType, ft2.paramType)
      const returnSub = this.unify(
        paramSub.apply(ft1.returnType),
        paramSub.apply(ft2.returnType)
      )

      return paramSub.compose(returnSub)
    }

    // ジェネリック型の場合
    if (type1.kind === "GenericType" && type2.kind === "GenericType") {
      const gt1 = type1 as AST.GenericType
      const gt2 = type2 as AST.GenericType

      // 特殊ケース: ArrayとListの相互変換を許可（内包表記のため）
      const isArrayListCompatible =
        (gt1.name === "Array" && gt2.name === "List") ||
        (gt1.name === "List" && gt2.name === "Array")

      if (
        !isArrayListCompatible &&
        (gt1.name !== gt2.name ||
          gt1.typeArguments.length !== gt2.typeArguments.length)
      ) {
        throw new Error(
          `Cannot unify ${typeToStringUtil(type1)} with ${typeToStringUtil(type2)}`
        )
      }

      let result = substitution
      for (let i = 0; i < gt1.typeArguments.length; i++) {
        const argSub = this.unify(
          result.apply(gt1.typeArguments[i]),
          result.apply(gt2.typeArguments[i])
        )
        result = result.compose(argSub)
      }

      return result
    }

    // Tuple型の場合
    if (type1.kind === "TupleType" && type2.kind === "TupleType") {
      const tt1 = type1 as AST.TupleType
      const tt2 = type2 as AST.TupleType

      // 要素数が一致する必要がある
      if (tt1.elementTypes.length !== tt2.elementTypes.length) {
        throw new Error(
          `Cannot unify ${typeToStringUtil(type1)} with ${typeToStringUtil(type2)}: different tuple lengths`
        )
      }

      // 各要素を統一
      let result = substitution
      for (let i = 0; i < tt1.elementTypes.length; i++) {
        const elementSub = this.unify(
          result.apply(tt1.elementTypes[i]),
          result.apply(tt2.elementTypes[i])
        )
        result = result.compose(elementSub)
      }

      return result
    }

    // Record型の場合 - 解決後の型で処理
    if (
      resolvedType1.kind === "RecordType" &&
      resolvedType2.kind === "RecordType"
    ) {
      const rt1 = resolvedType1 as AST.RecordType
      const rt2 = resolvedType2 as AST.RecordType

      // 構造的部分型関係をチェック：rt1 <: rt2 または rt2 <: rt1
      if (this.isSubtype(rt1, rt2)) {
        // rt1は rt2の部分型：共通フィールドを統一
        let result = substitution
        for (const superField of rt2.fields) {
          const subField = rt1.fields.find((f) => f.name === superField.name)
          if (subField) {
            const fieldSub = this.unify(
              result.apply(subField.type),
              result.apply(superField.type)
            )
            result = result.compose(fieldSub)
          }
        }
        return result
      }

      if (this.isSubtype(rt2, rt1)) {
        // rt2は rt1の部分型：共通フィールドを統一
        let result = substitution
        for (const superField of rt1.fields) {
          const subField = rt2.fields.find((f) => f.name === superField.name)
          if (subField) {
            const fieldSub = this.unify(
              result.apply(subField.type),
              result.apply(superField.type)
            )
            result = result.compose(fieldSub)
          }
        }
        return result
      }

      // 既存の構造的サブセット処理をバックアップとして保持
      // 長いレコードの方を基準にして、短いレコードがサブセットかチェック
      const [largerRecord, smallerRecord] =
        rt1.fields.length >= rt2.fields.length ? [rt1, rt2] : [rt2, rt1]
      const isSubset = isRecordSubsetUtil(smallerRecord, largerRecord)

      if (isSubset) {
        // サブセット関係がある場合、共通フィールドを統一
        let result = substitution
        for (const smallerField of smallerRecord.fields) {
          const largerField = largerRecord.fields.find(
            (f) => f.name === smallerField.name
          )
          if (largerField) {
            const fieldSub = this.unify(
              result.apply(smallerField.type),
              result.apply(largerField.type)
            )
            result = result.compose(fieldSub)
          }
        }
        return result
      }

      // サブセット関係がない場合、完全一致が必要
      if (rt1.fields.length !== rt2.fields.length) {
        throw new Error(
          `Cannot unify ${typeToStringUtil(type1)} with ${typeToStringUtil(type2)}: incompatible record structures`
        )
      }

      // フィールド名でソートして比較
      const fields1 = [...rt1.fields].sort((a, b) =>
        a.name.localeCompare(b.name)
      )
      const fields2 = [...rt2.fields].sort((a, b) =>
        a.name.localeCompare(b.name)
      )

      let result = substitution
      for (let i = 0; i < fields1.length; i++) {
        if (fields1[i].name !== fields2[i].name) {
          throw new Error(
            `Cannot unify ${typeToStringUtil(type1)} with ${typeToStringUtil(type2)}: field names don't match`
          )
        }

        const fieldSub = this.unify(
          result.apply(fields1[i].type),
          result.apply(fields2[i].type)
        )
        result = result.compose(fieldSub)
      }

      return result
    }

    // Struct型の場合
    if (type1.kind === "StructType" && type2.kind === "StructType") {
      const st1 = type1 as AST.StructType
      const st2 = type2 as AST.StructType

      // 同じ名前の構造体型でなければならない
      if (st1.name !== st2.name) {
        throw new Error(`Cannot unify struct types ${st1.name} and ${st2.name}`)
      }

      return substitution
    }

    // Struct型とRecord型の統一（構造的型付け）
    if (
      (type1.kind === "StructType" && type2.kind === "RecordType") ||
      (type1.kind === "RecordType" && type2.kind === "StructType")
    ) {
      const structType =
        type1.kind === "StructType"
          ? (type1 as AST.StructType)
          : (type2 as AST.StructType)
      const recordType =
        type1.kind === "RecordType"
          ? (type1 as AST.RecordType)
          : (type2 as AST.RecordType)

      // レコード型が構造体のフィールドのサブセットかチェック
      const structAsRecord = new AST.RecordType(
        structType.fields,
        structType.line,
        structType.column
      )
      if (isRecordSubsetUtil(recordType, structAsRecord)) {
        // 共通フィールドの型を統一
        let result = substitution
        for (const recordField of recordType.fields) {
          const structField = structType.fields.find(
            (f) => f.name === recordField.name
          )
          if (structField) {
            const fieldSub = this.unify(
              result.apply(recordField.type),
              result.apply(structField.type)
            )
            result = result.compose(fieldSub)
          }
        }
        return result
      }

      throw new Error(
        `Cannot unify ${typeToStringUtil(type1)} with ${typeToStringUtil(type2)}`
      )
    }

    // Record型と他の型の部分的統一（構造的部分型）
    if (type1.kind === "RecordType" || type2.kind === "RecordType") {
      // どちらか一方がRecordTypeの場合、構造的部分型をチェック
      const recordType =
        type1.kind === "RecordType"
          ? (type1 as AST.RecordType)
          : (type2 as AST.RecordType)
      const otherType = type1.kind === "RecordType" ? type2 : type1

      if (otherType.kind === "TypeVariable") {
        // 型変数の場合は通常の統一を行う
        const tv = otherType as TypeVariable
        if (occursCheckUtil(tv.id, recordType)) {
          throw new Error(
            `Infinite type: ${tv.name} occurs in ${typeToStringUtil(recordType)}`
          )
        }
        substitution.set(tv.id, recordType)
        return substitution
      }

      // その他の場合は統一不可能
      throw new Error(
        `Cannot unify ${typeToStringUtil(type1)} with ${typeToStringUtil(type2)}`
      )
    }

    // Union型の場合
    if (type1.kind === "UnionType" || type2.kind === "UnionType") {
      return this.unifyUnionTypes(type1, type2)
    }

    // Intersection型の場合
    if (
      type1.kind === "IntersectionType" ||
      type2.kind === "IntersectionType"
    ) {
      return this.unifyIntersectionTypes(type1, type2)
    }

    throw new Error(
      `Cannot unify ${typeToStringUtil(type1)} with ${typeToStringUtil(type2)}`
    )
  }

  // Union型の構造的等価性をチェック
  private areUnionTypesStructurallyEqual(
    union1: AST.UnionType,
    union2: AST.UnionType
  ): boolean {
    if (union1.types.length !== union2.types.length) {
      return false
    }

    // 各型を正規化して比較
    const normalizedTypes1 = union1.types
      .map((t) => this.normalizeType(t))
      .sort()
    const normalizedTypes2 = union2.types
      .map((t) => this.normalizeType(t))
      .sort()

    return normalizedTypes1.every((type1, i) => {
      const type2 = normalizedTypes2[i]
      return this.areTypesStructurallyEqual(type1, type2)
    })
  }

  // 型の正規化（型エイリアスを解決し、一貫した形式に変換）
  private normalizeType(type: AST.Type): string {
    const resolved = this.resolveTypeAlias(type)
    return typeToCanonicalStringUtil(resolved)
  }

  // 構造的等価性のチェック
  private areTypesStructurallyEqual(type1: string, type2: string): boolean {
    return type1 === type2
  }

  // Union型の統合
  private unifyUnionTypes(type1: AST.Type, type2: AST.Type): TypeSubstitution {
    const substitution = new TypeSubstitution()

    // 型エイリアスを解決してから処理
    const resolvedType1 = this.resolveTypeAlias(type1)
    const resolvedType2 = this.resolveTypeAlias(type2)

    // 両方がUnion型の場合
    if (
      resolvedType1.kind === "UnionType" &&
      resolvedType2.kind === "UnionType"
    ) {
      const union1 = resolvedType1 as AST.UnionType
      const union2 = resolvedType2 as AST.UnionType

      // 型集合が同じかチェック（順序は関係なし）
      if (union1.types.length === union2.types.length) {
        // より深い構造比較を実行
        if (this.areUnionTypesStructurallyEqual(union1, union2)) {
          // 同じユニオン型なので統一成功
          return substitution
        }
      }

      // 同じ型集合でない場合でも、各要素を個別に統合を試みる
      // これは型エイリアスが解決されたUnion型と新しく作成されたUnion型の統合で重要
      if (union1.types.length === union2.types.length) {
        // 順序に関係なく統合を試みる
        let result = substitution
        const usedIndices = new Set<number>()

        for (let i = 0; i < union1.types.length; i++) {
          let found = false
          for (let j = 0; j < union2.types.length; j++) {
            if (usedIndices.has(j)) continue

            try {
              const memberSub = this.unify(union1.types[i], union2.types[j])
              result = result.compose(memberSub)
              usedIndices.add(j)
              found = true
              break
            } catch {
              // この組み合わせは失敗、次を試す
            }
          }

          if (!found) {
            // 対応する型が見つからない場合は統合失敗
            throw new Error(
              `Cannot match union type member ${typeToStringUtil(union1.types[i])}`
            )
          }
        }

        return result
      }
    }

    // 型変数とユニオン型の場合、型変数をユニオン型に束縛
    if (type1.kind === "TypeVariable" && resolvedType2.kind === "UnionType") {
      const tv = type1 as TypeVariable
      substitution.set(tv.id, resolvedType2)
      return substitution
    }

    if (type2.kind === "TypeVariable" && resolvedType1.kind === "UnionType") {
      const tv = type2 as TypeVariable
      substitution.set(tv.id, resolvedType1)
      return substitution
    }

    // 片方がUnion型の場合
    // ADTコンストラクタ（PrimitiveType）とADT型（UnionType）の統合をチェック
    if (
      resolvedType1.kind === "UnionType" &&
      resolvedType2.kind === "PrimitiveType"
    ) {
      const union = resolvedType1 as AST.UnionType
      const primitive = resolvedType2 as AST.PrimitiveType

      // PrimitiveTypeがUnion型の構成要素かチェック
      const isConstructor = union.types.some(
        (memberType) =>
          memberType.kind === "PrimitiveType" &&
          (memberType as AST.PrimitiveType).name === primitive.name
      )

      if (isConstructor) {
        return substitution
      } else {
      }
    }

    if (
      resolvedType2.kind === "UnionType" &&
      resolvedType1.kind === "PrimitiveType"
    ) {
      const union = resolvedType2 as AST.UnionType
      const primitive = resolvedType1 as AST.PrimitiveType

      // PrimitiveTypeがUnion型の構成要素かチェック
      const isConstructor = union.types.some(
        (memberType) =>
          memberType.kind === "PrimitiveType" &&
          (memberType as AST.PrimitiveType).name === primitive.name
      )

      if (isConstructor) {
        return substitution
      } else {
      }
    }

    // Union型は基本的にその構成要素の型には割り当てできない
    // ただし、もう一方が型変数の場合は例外（型変数をUnion型に束縛）
    if (resolvedType1.kind === "UnionType" && type2.kind !== "TypeVariable") {
      const union1 = resolvedType1 as AST.UnionType

      // 特殊ケース: Union型の全ての要素が目標の型と統一可能な場合のみ許可
      // 例: Maybe<String> | Maybe<t1000> と Maybe<String> の統一（型変数を含む場合）
      // ただし、String | Int と String のような場合は許可しない
      let resultSubstitution = substitution
      let allCanUnify = true
      let hasTypeVariable = false

      // Union型の要素をチェック
      for (const memberType of union1.types) {
        if (
          memberType.kind === "TypeVariable" ||
          memberType.kind === "PolymorphicTypeVariable" ||
          (memberType.kind === "GenericType" &&
            (memberType as AST.GenericType).typeArguments.some(
              (arg) =>
                arg.kind === "TypeVariable" ||
                arg.kind === "PolymorphicTypeVariable"
            ))
        ) {
          hasTypeVariable = true
        }

        try {
          const memberSub = this.unify(memberType, resolvedType2)
          resultSubstitution = resultSubstitution.compose(memberSub)
        } catch {
          allCanUnify = false
          break
        }
      }

      // 型変数を含み、全ての要素が統一可能な場合のみ成功
      if (hasTypeVariable && allCanUnify) {
        return resultSubstitution
      }

      // Union型を非Union型に統合する場合
      // 例外1: type2もUnion型で、type1のすべてのメンバーがtype2に含まれる場合
      if (resolvedType2.kind === "UnionType") {
        const union2 = resolvedType2 as AST.UnionType
        // union1のすべてのメンバーがunion2に含まれているかチェック
        const allMembersIncluded = union1.types.every((member1) =>
          union2.types.some((member2) => typesEqualUtil(member1, member2))
        )
        if (allMembersIncluded) {
          return substitution
        }
      }

      // 例外2: type2が非Union型で、type1のメンバーのひとつがtype2と統合可能な場合
      if (resolvedType2.kind !== "UnionType") {
        // Union型のメンバーのひとつがtype2と統合可能かチェック
        const canUnifyWithMember = union1.types.some((memberType) => {
          try {
            this.unify(memberType, resolvedType2)
            return true
          } catch {
            return false
          }
        })
        if (canUnifyWithMember) {
          return substitution
        }
      }
      throw new Error(
        `Union type ${typeToStringUtil(resolvedType1)} cannot be assigned to ${typeToStringUtil(resolvedType2)}`
      )
    }

    if (resolvedType2.kind === "UnionType" && type1.kind !== "TypeVariable") {
      // 非Union型をUnion型に統合する場合は、非Union型がUnion型の構成要素である必要がある
      const union2 = resolvedType2 as AST.UnionType
      const isMember = union2.types.some((memberType) => {
        try {
          this.unify(resolvedType1, memberType)
          return true
        } catch (e) {
          return false
        }
      })
      if (isMember) {
        return substitution
      }
      throw new Error(
        `Type ${typeToStringUtil(resolvedType1)} is not assignable to union type ${typeToStringUtil(resolvedType2)}`
      )
    }

    throw new Error(
      `Cannot unify union types ${typeToStringUtil(type1)} with ${typeToStringUtil(type2)}`
    )
  }

  // Intersection型の統合
  private unifyIntersectionTypes(
    type1: AST.Type,
    type2: AST.Type
  ): TypeSubstitution {
    const substitution = new TypeSubstitution()

    // 両方がIntersection型の場合
    if (
      type1.kind === "IntersectionType" &&
      type2.kind === "IntersectionType"
    ) {
      const intersect1 = type1 as AST.IntersectionType
      const intersect2 = type2 as AST.IntersectionType

      // Intersection型をRecord型に変換してから統合
      const record1 = this.expandIntersectionToRecord(intersect1)
      const record2 = this.expandIntersectionToRecord(intersect2)

      if (record1 && record2) {
        return this.unify(record1, record2)
      }

      // Record型への変換が失敗した場合は従来の方法
      let result = substitution
      for (const member1 of intersect1.types) {
        for (const member2 of intersect2.types) {
          try {
            const sub = this.unify(member1, member2)
            result = result.compose(sub)
          } catch {
            // 統合失敗時は次の組み合わせを試す
          }
        }
      }
      return result
    }

    // 片方がIntersection型の場合
    if (type1.kind === "IntersectionType") {
      const intersect1 = type1 as AST.IntersectionType

      // Intersection型をRecord型に変換してから統合
      const record1 = this.expandIntersectionToRecord(intersect1)
      if (record1) {
        return this.unify(record1, type2)
      }

      // Record型への変換が失敗した場合は従来の方法
      let result = substitution
      for (const memberType of intersect1.types) {
        const sub = this.unify(memberType, type2)
        result = result.compose(sub)
      }
      return result
    }

    if (type2.kind === "IntersectionType") {
      const intersect2 = type2 as AST.IntersectionType

      // Intersection型をRecord型に変換してから統合
      const record2 = this.expandIntersectionToRecord(intersect2)
      if (record2) {
        return this.unify(type1, record2)
      }

      // Record型への変換が失敗した場合は従来の方法
      let result = substitution
      for (const memberType of intersect2.types) {
        const sub = this.unify(type1, memberType)
        result = result.compose(sub)
      }
      return result
    }

    throw new Error(
      `Cannot unify intersection types ${typeToStringUtil(type1)} with ${typeToStringUtil(type2)}`
    )
  }

  // Intersection型をRecord型に展開
  private expandIntersectionToRecord(
    intersectionType: AST.IntersectionType
  ): AST.RecordType | null {
    const mergedFields: AST.RecordField[] = []

    for (const memberType of intersectionType.types) {
      // 型エイリアスを解決
      const resolvedType = this.resolveTypeAlias(memberType)

      if (resolvedType.kind === "RecordType") {
        const recordType = resolvedType as AST.RecordType

        // フィールドをマージ
        for (const field of recordType.fields) {
          // 同じ名前のフィールドが既に存在するかチェック
          const existingField = mergedFields.find((f) => f.name === field.name)
          if (existingField) {
            // 同じ名前のフィールドが異なる型を持つ場合はエラー
            const existingTypeName = getTypeNameUtil(existingField.type)
            const newTypeName = getTypeNameUtil(field.type)
            if (existingTypeName !== newTypeName) {
              throw new Error(
                `Field '${field.name}' has conflicting types in intersection: ${existingTypeName} and ${newTypeName}`
              )
            }
          } else {
            mergedFields.push(field)
          }
        }
      } else {
        // Record型でない場合はRecord型への変換失敗
        return null
      }
    }

    // マージされたフィールドでRecord型を作成
    return new AST.RecordType(
      mergedFields,
      intersectionType.line,
      intersectionType.column
    )
  }

  // 型エイリアスの解決（循環参照対応版）
  private resolveTypeAlias(
    type: AST.Type,
    visited: Set<string> = new Set()
  ): AST.Type {
    if (type.kind === "PrimitiveType") {
      const pt = type as AST.PrimitiveType

      // 循環参照チェック
      if (visited.has(pt.name)) {
        return type // 循環を検出したら元の型を返す
      }

      // 現在の環境で型エイリアスをチェック
      for (const [name, aliasedType] of this.currentEnvironment) {
        if (name === pt.name) {
          // 訪問済みに追加して再帰的にエイリアスを解決
          const newVisited = new Set(visited)
          newVisited.add(pt.name)
          return this.resolveTypeAlias(aliasedType, newVisited)
        }
      }
    } else if (type.kind === "GenericType") {
      // ジェネリック型の場合（例: Box<Int>）
      const genericType = type as AST.GenericType

      // 循環参照チェック
      if (visited.has(genericType.name)) {
        return type
      }

      // まず組み込み型をチェック
      const builtinTypes = [
        "Maybe",
        "Either",
        "List",
        "Array",
        "Signal",
        "Task",
      ]
      if (builtinTypes.includes(genericType.name)) {
        // 組み込み型は型引数を解決して返す
        const resolvedTypeArgs = genericType.typeArguments.map((arg) =>
          this.resolveTypeAlias(arg, visited)
        )
        return new AST.GenericType(
          genericType.name,
          resolvedTypeArgs,
          genericType.line,
          genericType.column
        )
      }

      // ジェネリック型エイリアスをチェック
      const typeAlias = this.typeAliases.get(genericType.name)
      if (
        typeAlias?.kind === "TypeAliasDeclaration" &&
        (typeAlias as AST.TypeAliasDeclaration).typeParameters
      ) {
        const genericAlias = typeAlias as AST.TypeAliasDeclaration
        // 型パラメータと型引数のマッピングを作成
        const typeParameterMap = new Map<string, AST.Type>()
        for (
          let i = 0;
          i < genericAlias.typeParameters.length &&
          i < genericType.typeArguments.length;
          i++
        ) {
          const param = genericAlias.typeParameters[i]
          const arg = genericType.typeArguments[i]
          typeParameterMap.set(param.name, arg)
        }

        // 型変数名で置換する（型変数名がパラメータ名と異なる場合のため）
        const instantiatedType = this.substituteTypeVariablesInGenericAlias(
          genericAlias.aliasedType,
          genericAlias.typeParameters,
          genericType.typeArguments
        )

        const newVisited = new Set(visited)
        newVisited.add(genericType.name)
        return this.resolveTypeAlias(instantiatedType, newVisited)
      }
    } else if (type.kind === "IntersectionType") {
      // Intersection型の場合、各構成要素を再帰的に解決
      const intersectionType = type as AST.IntersectionType
      const resolvedTypes = intersectionType.types.map((t) =>
        this.resolveTypeAlias(t, visited)
      )

      // 全ての構成要素がRecord型の場合、Record型に統合
      if (resolvedTypes.every((t) => t.kind === "RecordType")) {
        return mergeRecordTypesUtil(resolvedTypes as AST.RecordType[])
      }

      // そうでない場合は、解決された型でIntersection型を再構築
      return new AST.IntersectionType(resolvedTypes, type.line, type.column)
    } else if (type.kind === "UnionType") {
      // Union型の場合、各構成要素を再帰的に解決
      const unionType = type as AST.UnionType
      const resolvedTypes = unionType.types.map((t) =>
        this.resolveTypeAlias(t, visited)
      )
      return new AST.UnionType(resolvedTypes, type.line, type.column)
    }
    return type
  }

  // 環境を保持するためのフィールド（既に存在する可能性）
  private currentEnvironment: Map<string, AST.Type> = new Map()

  // 型エイリアス内の型パラメータと型変数の対応関係を保持
  private typeAliasParameterMappings: Map<string, Map<string, string>> =
    new Map()

  /**
   * 部分型関係を判定: subType <: superType
   * subTypeがsuperTypeの部分型（サブタイプ）であるかを判定
   */
  private isSubtype(subType: AST.Type, superType: AST.Type): boolean {
    // 同じ型なら部分型関係成立
    if (typesEqualUtil(subType, superType)) {
      return true
    }

    // 型エイリアスを解決
    const resolvedSub = this.resolveTypeAlias(subType)
    const resolvedSuper = this.resolveTypeAlias(superType)

    // 解決後に同じ型なら部分型関係成立
    if (typesEqualUtil(resolvedSub, resolvedSuper)) {
      return true
    }

    // Record型の構造的部分型関係をチェック
    if (
      resolvedSub.kind === "RecordType" &&
      resolvedSuper.kind === "RecordType"
    ) {
      return this.isRecordSubtype(
        resolvedSub as AST.RecordType,
        resolvedSuper as AST.RecordType
      )
    }

    // その他の型の部分型関係（将来の拡張ポイント）
    return false
  }

  /**
   * Record型の構造的部分型関係を判定 (type-comparison.tsに委譲)
   */
  private isRecordSubtype(
    subRecord: AST.RecordType,
    superRecord: AST.RecordType
  ): boolean {
    return isRecordSubtypeUtil(subRecord, superRecord, (sub, sup) =>
      this.isSubtype(sub, sup)
    )
  }

  // メソッド呼び出しの型推論
  private generateConstraintsForMethodCall(
    call: AST.MethodCall,
    env: Map<string, AST.Type>
  ): AST.Type {
    // レシーバーの型を推論
    const receiverType = this.generateConstraintsForExpression(
      call.receiver,
      env
    )

    // 引数の型を推論
    const argTypes: AST.Type[] = []
    for (const arg of call.arguments) {
      argTypes.push(this.generateConstraintsForExpression(arg, env))
    }

    // レシーバーの型名を取得
    let receiverTypeName: string | null = null
    if (receiverType.kind === "StructType") {
      receiverTypeName = (receiverType as AST.StructType).name
    } else if (receiverType.kind === "PrimitiveType") {
      receiverTypeName = (receiverType as AST.PrimitiveType).name
    } else if (receiverType.kind === "TypeVariable") {
      // 型変数の場合、nodeTypeMapから解決を試みる
      for (const [_node, type] of this.nodeTypeMap.entries()) {
        if (type === receiverType && type.kind === "StructType") {
          receiverTypeName = (type as AST.StructType).name
          break
        }
      }
    }

    // implブロックからメソッドを検索
    let methodReturnType: AST.Type | null = null
    if (receiverTypeName) {
      const methodKey = `${receiverTypeName}.${call.methodName}`
      const methodInfo = this.methodEnvironment.get(methodKey)

      if (methodInfo && methodInfo.kind === "MethodDeclaration") {
        const method = methodInfo as AST.MethodDeclaration

        // メソッドの戻り値型を取得
        if (method.returnType) {
          methodReturnType = method.returnType

          // 引数の型チェック（selfを除く）
          const methodParams = method.parameters.filter(
            (p) => !p.isImplicitSelf
          )
          if (methodParams.length === argTypes.length) {
            // 各引数の型制約を追加
            for (let i = 0; i < methodParams.length; i++) {
              this.constraints.push(
                new TypeConstraint(
                  argTypes[i],
                  methodParams[i].type,
                  call.line,
                  call.column,
                  `Method argument ${i} type mismatch`
                )
              )
            }
          }

          // レシーバー型制約を追加
          this.constraints.push(
            new TypeConstraint(
              receiverType,
              new AST.StructType(receiverTypeName, [], call.line, call.column),
              call.line,
              call.column,
              `Method receiver type mismatch`
            )
          )
        }
      }
    }

    // メソッドが見つからない場合は新しい型変数を使用
    if (!methodReturnType) {
      methodReturnType = this.freshTypeVariable(call.line, call.column)

      // カリー化されたFunction型として制約を構築（従来の方法）
      let expectedMethodType: AST.Type = methodReturnType

      for (let i = argTypes.length - 1; i >= 0; i--) {
        expectedMethodType = new AST.FunctionType(
          argTypes[i],
          expectedMethodType,
          call.line,
          call.column
        )
      }

      expectedMethodType = new AST.FunctionType(
        receiverType,
        expectedMethodType,
        call.line,
        call.column
      )

      this.constraints.push(
        new TypeConstraint(
          expectedMethodType,
          expectedMethodType,
          call.line,
          call.column,
          `Method call ${call.methodName} on type ${formatTypeUtil(receiverType)}`
        )
      )
    }

    // MethodCallノードと型を関連付け（LSP hover用）
    this.nodeTypeMap.set(call, methodReturnType)

    return methodReturnType
  }

  private generateConstraintsForTupleExpression(
    tuple: AST.TupleExpression,
    env: Map<string, AST.Type>
  ): AST.Type {
    // 各要素の型を推論
    const elementTypes: AST.Type[] = []

    for (const element of tuple.elements) {
      const elementType = this.generateConstraintsForExpression(element, env)
      elementTypes.push(elementType)
    }

    // タプル型を作成
    return new AST.TupleType(elementTypes, tuple.line, tuple.column)
  }

  private generateConstraintsForStructDeclaration(
    structDecl: AST.StructDeclaration,
    env: Map<string, AST.Type>
  ): void {
    // 構造体型を作成
    const structType = new AST.StructType(
      structDecl.name,
      structDecl.fields,
      structDecl.line,
      structDecl.column
    )

    // Debug: Log struct registration

    // 環境に構造体型を登録
    env.set(structDecl.name, structType)

    // nodeTypeMapにも登録
    this.nodeTypeMap.set(structDecl, structType)
  }

  private generateConstraintsForImplBlock(
    implBlock: AST.ImplBlock,
    env: Map<string, AST.Type>
  ): void {
    // impl ブロックの型名が存在するかチェック
    const implType = env.get(implBlock.typeName)
    if (!implType) {
      this.errors.push(
        new TypeInferenceError(
          `Unknown type for impl block: ${implBlock.typeName}`,
          implBlock.line,
          implBlock.column
        )
      )
      return
    }

    // メソッドの制約を生成
    for (const method of implBlock.methods) {
      this.generateConstraintsForMethodDeclaration(method, env, implType)
    }

    // 演算子の制約を生成
    for (const operator of implBlock.operators) {
      this.generateConstraintsForOperatorDeclaration(operator, env, implType)
    }

    // モノイドの制約を生成
    if (implBlock.monoid) {
      this.generateConstraintsForMonoidDeclaration(
        implBlock.monoid,
        env,
        implType
      )
    }
  }

  private generateConstraintsForMethodDeclaration(
    method: AST.MethodDeclaration,
    env: Map<string, AST.Type>,
    implType: AST.Type
  ): void {
    this.registerMethodInEnvironment(method, implType)

    // パラメータの型を先に解決してからFunctionTypeを作成
    const resolvedParameters = method.parameters.map((param) => ({
      ...param,
      type: this.resolveParameterType(param, implType, env),
    }))

    const functionType = this.buildFunctionType(
      resolvedParameters,
      method.returnType
    )

    env.set(`${method.name}`, functionType)
    this.nodeTypeMap.set(method, functionType)

    const methodEnv = this.createMethodEnvironment(env, implType)
    this.addMethodParametersToEnvironment(method, methodEnv, implType, env)

    const bodyType = this.generateConstraintsForExpression(
      method.body,
      methodEnv
    )
    this.addMethodBodyTypeConstraint(method, bodyType)
  }

  private registerMethodInEnvironment(
    method: AST.MethodDeclaration,
    implType: AST.Type
  ): void {
    if (implType.kind === "StructType") {
      const structType = implType as AST.StructType
      const methodKey = `${structType.name}.${method.name}`
      this.methodEnvironment.set(methodKey, method)
    }
  }

  private createMethodEnvironment(
    env: Map<string, AST.Type>,
    implType: AST.Type
  ): Map<string, AST.Type> {
    const methodEnv = new Map(env)

    if (implType.kind === "StructType") {
      const structType = implType as AST.StructType
      methodEnv.set(structType.name, implType)
    }

    this.addStructTypesToEnvironment(env, methodEnv)
    return methodEnv
  }

  private addMethodParametersToEnvironment(
    method: AST.MethodDeclaration,
    methodEnv: Map<string, AST.Type>,
    implType: AST.Type,
    env: Map<string, AST.Type>
  ): void {
    for (const param of method.parameters) {
      const resolvedType = this.resolveParameterType(param, implType, env)
      methodEnv.set(param.name, resolvedType)
    }
  }

  private addMethodBodyTypeConstraint(
    method: AST.MethodDeclaration,
    bodyType: AST.Type
  ): void {
    this.addConstraint(
      new TypeConstraint(
        bodyType,
        method.returnType,
        method.line,
        method.column,
        `Method ${method.name} body type`
      )
    )
  }

  private generateConstraintsForOperatorDeclaration(
    operator: AST.OperatorDeclaration,
    env: Map<string, AST.Type>,
    implType: AST.Type
  ): void {
    // パラメータの型を先に解決してからFunctionTypeを作成
    const resolvedParameters = operator.parameters.map((param) => ({
      ...param,
      type: this.resolveParameterType(param, implType, env),
    }))

    const functionType = this.buildFunctionType(
      resolvedParameters,
      operator.returnType
    )
    this.registerOperatorInEnvironment(operator, functionType, env)

    const operatorEnv = this.createOperatorEnvironment(env, implType)
    this.addParametersToEnvironment(operator, operatorEnv, implType, env)

    const bodyType = this.generateConstraintsForExpression(
      operator.body,
      operatorEnv
    )
    this.addBodyTypeConstraint(operator, bodyType)
  }

  private registerOperatorInEnvironment(
    operator: AST.OperatorDeclaration,
    functionType: AST.Type,
    env: Map<string, AST.Type>
  ): void {
    const generalizedOperatorType = this.generalize(functionType, env)
    env.set(`${operator.operator}`, generalizedOperatorType)
    this.nodeTypeMap.set(operator, generalizedOperatorType)
  }

  private createOperatorEnvironment(
    env: Map<string, AST.Type>,
    implType: AST.Type
  ): Map<string, AST.Type> {
    const operatorEnv = new Map(env)

    if (implType.kind === "StructType") {
      const structType = implType as AST.StructType
      operatorEnv.set(structType.name, implType)
    }

    this.addStructTypesToEnvironment(env, operatorEnv)
    return operatorEnv
  }

  private addStructTypesToEnvironment(
    sourceEnv: Map<string, AST.Type>,
    targetEnv: Map<string, AST.Type>
  ): void {
    for (const [key, value] of sourceEnv.entries()) {
      if (value.kind === "StructType") {
        targetEnv.set(key, value)
      }
    }
  }

  private addParametersToEnvironment(
    operator: AST.OperatorDeclaration,
    operatorEnv: Map<string, AST.Type>,
    implType: AST.Type,
    env: Map<string, AST.Type>
  ): void {
    for (const param of operator.parameters) {
      const resolvedType = this.resolveParameterType(param, implType, env)
      operatorEnv.set(param.name, resolvedType)
      this.nodeTypeMap.set(param, resolvedType)
    }
  }

  private resolveParameterType(
    param: AST.Parameter,
    implType: AST.Type,
    env: Map<string, AST.Type>
  ): AST.Type {
    if (param.isImplicitSelf || param.isImplicitOther) {
      param.type = implType
      return implType
    }

    const resolvedType = this.resolveTypeAlias(param.type)
    return this.resolveStructTypeFromEnvironment(resolvedType, env)
  }

  private resolveStructTypeFromEnvironment(
    resolvedType: AST.Type,
    env: Map<string, AST.Type>
  ): AST.Type {
    if (resolvedType.kind === "PrimitiveType") {
      const structTypeFromEnv = env.get(
        (resolvedType as AST.PrimitiveType).name
      )
      if (structTypeFromEnv?.kind === "StructType") {
        return structTypeFromEnv
      }
    }
    return resolvedType
  }

  private addBodyTypeConstraint(
    operator: AST.OperatorDeclaration,
    bodyType: AST.Type
  ): void {
    this.addConstraint(
      new TypeConstraint(
        bodyType,
        operator.returnType,
        operator.line,
        operator.column,
        `Operator ${operator.operator} body type`
      )
    )
  }

  private generateConstraintsForMonoidDeclaration(
    monoid: AST.MonoidDeclaration,
    env: Map<string, AST.Type>,
    implType: AST.Type
  ): void {
    // identity値の制約を生成
    const identityType = this.generateConstraintsForExpression(
      monoid.identity,
      env
    )

    // identity値は型と一致する必要がある
    this.addConstraint(
      new TypeConstraint(
        identityType,
        implType,
        monoid.line,
        monoid.column,
        "Monoid identity type"
      )
    )

    // 演算子の制約を生成
    this.generateConstraintsForOperatorDeclaration(
      monoid.operator,
      env,
      implType
    )
  }

  // 型が構造体型または構造体型に解決される型変数かをチェック
  private isStructOrResolvesToStruct(
    type: AST.Type,
    env: Map<string, AST.Type>
  ): boolean {
    // 直接的に構造体型の場合
    if (type.kind === "StructType") {
      return true
    }

    // プリミティブ型の場合、環境から構造体型を検索
    if (type.kind === "PrimitiveType") {
      const resolved = env.get((type as AST.PrimitiveType).name)
      return resolved?.kind === "StructType"
    }

    // 型変数の場合、現在の型置換を確認
    // 注意: 制約生成段階では型変数は未解決のため、構造体型の可能性として扱う
    if (type.kind === "TypeVariable") {
      return true // 保守的に true を返し、構造体の可能性を考慮
    }

    return false
  }

  // パラメータリストから関数シグネチャを構築（カリー化）
  private buildFunctionType(
    parameters: AST.Parameter[],
    returnType: AST.Type
  ): AST.Type {
    if (parameters.length === 0) {
      return returnType
    }

    // 右結合でカリー化されたFunction型を構築
    let result = returnType
    for (let i = parameters.length - 1; i >= 0; i--) {
      result = new AST.FunctionType(
        parameters[i].type,
        result,
        parameters[i].line,
        parameters[i].column
      )
    }

    return result
  }

  private generateConstraintsForStructExpression(
    structExpr: AST.StructExpression,
    env: Map<string, AST.Type>,
    _expectedType?: AST.Type
  ): AST.Type {
    // 構造体型を環境から取得
    const structType = env.get(structExpr.structName)

    if (!structType) {
      this.errors.push(
        new TypeInferenceError(
          `Unknown struct type: ${structExpr.structName}`,
          structExpr.line,
          structExpr.column
        )
      )
      return this.freshTypeVariable(structExpr.line, structExpr.column)
    }

    if (structType.kind !== "StructType") {
      this.errors.push(
        new TypeInferenceError(
          `${structExpr.structName} is not a struct type`,
          structExpr.line,
          structExpr.column
        )
      )
      return this.freshTypeVariable(structExpr.line, structExpr.column)
    }

    // フィールドの型チェック
    const providedFieldMap = new Map<
      string,
      { field: AST.RecordInitField | AST.RecordSpreadField; type: AST.Type }
    >()

    // まずスプレッドフィールドを処理
    for (const field of structExpr.fields) {
      if (field.kind === "RecordSpreadField") {
        const spreadField = field as AST.RecordSpreadField
        const spreadType = this.generateConstraintsForExpression(
          spreadField.spreadExpression.expression,
          env
        )

        // スプレッド元が同じ構造体型であることを確認
        if (spreadType.kind === "StructType") {
          const sourceStruct = spreadType as AST.StructType
          // スプレッド元のフィールドをすべて追加
          for (const sourceField of sourceStruct.fields) {
            providedFieldMap.set(sourceField.name, {
              field: spreadField,
              type: sourceField.type,
            })
          }
        } else {
          this.errors.push(
            new TypeInferenceError(
              `Cannot spread non-struct type in struct literal`,
              spreadField.line,
              spreadField.column
            )
          )
        }
      }
    }

    // 次に明示的なフィールドで上書き
    for (const field of structExpr.fields) {
      if (field.kind === "RecordInitField") {
        const initField = field as AST.RecordInitField
        const fieldType = this.generateConstraintsForExpression(
          initField.value,
          env
        )
        providedFieldMap.set(initField.name, {
          field: initField,
          type: fieldType,
        })
      } else if (field.kind === "RecordShorthandField") {
        const shorthandField = field as AST.RecordShorthandField
        // 変数名と同じ名前の変数を環境から検索
        const variableType = env.get(shorthandField.name)
        if (!variableType) {
          this.errors.push(
            new TypeInferenceError(
              `Undefined variable '${shorthandField.name}' in shorthand property`,
              shorthandField.line,
              shorthandField.column
            )
          )
          // エラーの場合はTypeVariableをフォールバック
          const fallbackType = this.freshTypeVariable(
            shorthandField.line,
            shorthandField.column
          )
          providedFieldMap.set(shorthandField.name, {
            field: shorthandField,
            type: fallbackType,
          })
        } else {
          providedFieldMap.set(shorthandField.name, {
            field: shorthandField,
            type: variableType,
          })
        }
      }
    }

    // Maybe型フィールドの自動補完（Struct版）
    // オブジェクトベースのコンストラクタでは、デフォルト値はコンストラクタ内で適用されるため
    // ここでの自動補完は不要

    // 必要なフィールドがすべて提供されているかチェック
    for (const field of (structType as any).fields) {
      const providedData = providedFieldMap.get(field.name)

      if (!providedData) {
        // Maybe型フィールドは省略可能
        if (!isMaybeTypeUtil(field.type)) {
          this.errors.push(
            new TypeInferenceError(
              `Missing field '${field.name}' in struct ${structExpr.structName}`,
              structExpr.line,
              structExpr.column
            )
          )
        }
        continue
      }

      // フィールドの型と値の型が一致することを制約として追加
      this.constraints.push(
        new TypeConstraint(
          providedData.type,
          field.type,
          providedData.field.line,
          providedData.field.column,
          `Struct field ${field.name}`
        )
      )
    }

    // 余分なフィールドがないかチェック
    for (const [fieldName, _] of providedFieldMap) {
      if (!(structType as any).fields.find((f: any) => f.name === fieldName)) {
        this.errors.push(
          new TypeInferenceError(
            `Unknown field '${fieldName}' in struct ${structExpr.structName}`,
            structExpr.line,
            structExpr.column
          )
        )
      }
    }

    return structType
  }

  private generateConstraintsForMatchExpression(
    match: AST.MatchExpression,
    env: Map<string, AST.Type>
  ): AST.Type {
    // マッチ対象の式の型を推論
    const exprType = this.generateConstraintsForExpression(
      match.expression,
      env
    )

    if (match.cases.length === 0) {
      this.errors.push(
        new TypeInferenceError(
          "Match expression must have at least one case",
          match.line,
          match.column
        )
      )
      return this.freshTypeVariable(match.line, match.column)
    }

    // 各ケースの結果型を収集
    const caseResultTypes: AST.Type[] = []

    for (let i = 0; i < match.cases.length; i++) {
      const caseItem = match.cases[i]

      // パターンマッチングで新しい変数環境を作成
      const caseEnv = new Map(env)
      this.generateConstraintsForPattern(caseItem.pattern, exprType, caseEnv)

      // ケースの結果型を推論
      const caseResultType = this.generateConstraintsForExpression(
        caseItem.expression,
        caseEnv
      )

      caseResultTypes.push(caseResultType)
    }

    // 複数の結果型がある場合はUnion型として統合
    const resultType =
      caseResultTypes.length === 1
        ? caseResultTypes[0]
        : createFlattenedUnionTypeUtil(
            caseResultTypes,
            match.line,
            match.column
          )

    return resultType
  }

  private generateConstraintsForPattern(
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

        // リテラルタイプを使用（利用可能な場合）
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
              literalType = this.freshTypeVariable(pattern.line, pattern.column)
          }
        } else {
          // フォールバック：値の型から推論
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
              literalType = this.freshTypeVariable(pattern.line, pattern.column)
          }
        }

        this.addConstraint(
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

        // 環境からコンストラクタの型を取得
        const constructorType = env.get(ctorPattern.constructorName)
        if (!constructorType) {
          this.errors.push(
            new TypeInferenceError(
              `Unknown constructor: ${ctorPattern.constructorName}`,
              pattern.line,
              pattern.column
            )
          )
          return
        }

        // コンストラクタがADT型を返すことを確認
        let adtType = expectedType
        let currentType = constructorType
        const paramTypes: AST.Type[] = []

        // Function型を辿ってパラメータ型を抽出
        while (currentType instanceof AST.FunctionType) {
          paramTypes.push(currentType.paramType)
          currentType = currentType.returnType
        }

        // 最終的な戻り値型（ADT型）
        adtType = currentType

        // 期待する型とADT型が一致することを確認
        this.addConstraint(
          new TypeConstraint(
            expectedType,
            adtType,
            pattern.line,
            pattern.column,
            `Constructor pattern type`
          )
        )

        // パターンの引数と型パラメータの数が一致することを確認
        if (ctorPattern.patterns.length !== paramTypes.length) {
          this.errors.push(
            new TypeInferenceError(
              `Constructor ${ctorPattern.constructorName} expects ${paramTypes.length} arguments, but got ${ctorPattern.patterns.length}`,
              pattern.line,
              pattern.column
            )
          )
          return
        }

        // ネストしたパターンに対応する型で再帰的に処理
        for (let i = 0; i < ctorPattern.patterns.length; i++) {
          this.generateConstraintsForPattern(
            ctorPattern.patterns[i],
            paramTypes[i],
            env
          )
        }
        break
      }

      case "WildcardPattern":
        // ワイルドカードパターン: 何にでもマッチし、変数をバインドしない
        break

      case "OrPattern": {
        // orパターン: すべてのサブパターンが同じ型である必要がある
        const orPattern = pattern as AST.OrPattern

        // 各サブパターンに対して制約を生成
        for (const subPattern of orPattern.patterns) {
          this.generateConstraintsForPattern(subPattern, expectedType, env)
        }
        break
      }

      case "TuplePattern": {
        // タプルパターン: (x, y, z) = tuple
        const tuplePattern = pattern as AST.TuplePattern

        // タプル型の要素数と一致するかチェック
        const expectedElementTypes: AST.Type[] = []
        for (let i = 0; i < tuplePattern.patterns.length; i++) {
          expectedElementTypes.push(
            this.freshTypeVariable(pattern.line, pattern.column)
          )
        }

        const tupleType = new AST.TupleType(
          expectedElementTypes,
          pattern.line,
          pattern.column
        )
        this.addConstraint(
          new TypeConstraint(
            expectedType,
            tupleType,
            pattern.line,
            pattern.column,
            "Tuple pattern structure"
          )
        )

        // 各パターン要素に対応する型で再帰的に処理
        for (let i = 0; i < tuplePattern.patterns.length; i++) {
          this.generateConstraintsForPattern(
            tuplePattern.patterns[i],
            expectedElementTypes[i],
            env
          )
        }
        break
      }

      case "GuardPattern": {
        // ガードパターン: pattern when condition
        const guardPattern = pattern as AST.GuardPattern

        // 基底パターンの型制約を生成
        this.generateConstraintsForPattern(
          guardPattern.pattern,
          expectedType,
          env
        )

        // ガード条件の型制約を生成（Bool型である必要がある）
        const guardType = this.generateConstraintsForExpression(
          guardPattern.guard,
          env
        )
        const boolType = new AST.PrimitiveType(
          "Bool",
          pattern.line,
          pattern.column
        )

        this.addConstraint(
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
        // リスト糖衣構文パターン: `[x, y, ...rest]
        const listSugarPattern = pattern as AST.ListSugarPattern

        // List型であることを確認
        const listType = new AST.GenericType(
          "List",
          [this.freshTypeVariable(pattern.line, pattern.column)],
          pattern.line,
          pattern.column
        )

        this.addConstraint(
          new TypeConstraint(
            expectedType,
            listType,
            pattern.line,
            pattern.column,
            "List sugar pattern expects List type"
          )
        )

        // 各要素パターンに対して再帰的に制約を生成
        const elementType = listType.typeArguments[0]

        for (const elemPattern of listSugarPattern.patterns) {
          this.generateConstraintsForPattern(elemPattern, elementType, env)
        }

        // restパターンがある場合
        if (listSugarPattern.hasRest && listSugarPattern.restPattern) {
          // restはList型全体
          this.generateConstraintsForPattern(
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

        // Array型であることを確認
        const arrayType = new AST.GenericType(
          "Array",
          [this.freshTypeVariable(pattern.line, pattern.column)],
          pattern.line,
          pattern.column
        )

        this.addConstraint(
          new TypeConstraint(
            expectedType,
            arrayType,
            pattern.line,
            pattern.column,
            "Array pattern expects Array type"
          )
        )

        // 各要素パターンに対して再帰的に制約を生成
        const elementType = arrayType.typeArguments[0]

        for (const elemPattern of arrayPattern.patterns) {
          this.generateConstraintsForPattern(elemPattern, elementType, env)
        }

        // restパターンがある場合
        if (arrayPattern.hasRest && arrayPattern.restPattern) {
          // restはArray型全体
          this.generateConstraintsForPattern(
            arrayPattern.restPattern,
            expectedType,
            env
          )
        }

        break
      }

      default:
        this.errors.push(
          new TypeInferenceError(
            `Unsupported pattern type: ${pattern.kind}`,
            pattern.line,
            pattern.column
          )
        )
    }
  }

  private generateConstraintsForRecordExpression(
    record: AST.RecordExpression,
    env: Map<string, AST.Type>,
    expectedType?: AST.Type
  ): AST.Type {
    // Mapを使って重複フィールドを避ける（後から来たフィールドで上書き）
    const fieldMap = new Map<string, AST.RecordField>()

    // 期待される型からレコード型を取得
    let expectedRecordType: AST.RecordType | null = null
    if (expectedType) {
      const resolvedExpectedType = this.resolveTypeAlias(expectedType)
      if (resolvedExpectedType.kind === "RecordType") {
        expectedRecordType = resolvedExpectedType as AST.RecordType
      }
    }

    for (const field of record.fields) {
      if (field.kind === "RecordInitField") {
        const initField = field as AST.RecordInitField

        // 期待されるフィールド型を取得
        let expectedFieldType: AST.Type | undefined
        if (expectedRecordType) {
          const expectedField = expectedRecordType.fields.find(
            (f) => f.name === initField.name
          )
          if (expectedField) {
            expectedFieldType = expectedField.type
          }
        }

        let fieldType = this.generateConstraintsForExpression(
          initField.value,
          env,
          expectedFieldType
        )

        // 期待されるフィールド型がある場合の特殊処理
        if (expectedFieldType) {
          const resolvedExpectedFieldType =
            this.resolveTypeAlias(expectedFieldType)

          // 期待される型がUnion型の場合、フィールド型をUnion型に統一を試みる
          if (resolvedExpectedFieldType.kind === "UnionType") {
            const union = resolvedExpectedFieldType as AST.UnionType
            // フィールド型がUnion型の構成要素の一つかチェック
            const isMember = union.types.some((memberType) => {
              try {
                this.unify(fieldType, memberType)
                return true
              } catch {
                return false
              }
            })

            if (isMember) {
              // 統一可能な場合、期待される型（Union型）をフィールド型として使用
              fieldType = expectedFieldType
            }
          }
        }

        // 同名フィールドは上書き
        fieldMap.set(
          initField.name,
          new AST.RecordField(
            initField.name,
            fieldType,
            initField.line,
            initField.column
          )
        )
      } else if (field.kind === "RecordShorthandField") {
        const shorthandField = field as AST.RecordShorthandField
        // 変数名と同じ名前の変数を環境から検索
        const variableType = env.get(shorthandField.name)
        if (!variableType) {
          this.errors.push(
            new TypeInferenceError(
              `Undefined variable '${shorthandField.name}' in shorthand property`,
              shorthandField.line,
              shorthandField.column
            )
          )
          // エラーの場合はTypeVariableをフォールバック
          const fallbackType = this.freshTypeVariable(
            shorthandField.line,
            shorthandField.column
          )
          fieldMap.set(
            shorthandField.name,
            new AST.RecordField(
              shorthandField.name,
              fallbackType,
              shorthandField.line,
              shorthandField.column
            )
          )
        } else {
          fieldMap.set(
            shorthandField.name,
            new AST.RecordField(
              shorthandField.name,
              variableType,
              shorthandField.line,
              shorthandField.column
            )
          )
        }
      } else if (field.kind === "RecordSpreadField") {
        const spreadField = field as AST.RecordSpreadField
        const spreadType = this.generateConstraintsForExpression(
          spreadField.spreadExpression,
          env
        )

        // スプレッド元がレコード型であることを確認
        if (spreadType.kind === "RecordType") {
          const recordType = spreadType as AST.RecordType
          // スプレッド元のフィールドをマージ（後から来たフィールドで上書き）
          for (const sourceField of recordType.fields) {
            fieldMap.set(sourceField.name, sourceField)
          }
        } else {
          this.errors.push(
            new TypeInferenceError(
              `Cannot spread non-record type in record literal`,
              spreadField.line,
              spreadField.column
            )
          )
        }
      }
    }

    // Maybe型フィールドの自動補完
    if (
      expectedType &&
      (expectedType.kind === "RecordType" ||
        expectedType.kind === "PrimitiveType" ||
        expectedType.kind === "IntersectionType")
    ) {
      let expectedRecordType: AST.RecordType | null = null

      // 期待される型がRecord型の場合
      if (expectedType.kind === "RecordType") {
        expectedRecordType = expectedType as AST.RecordType
      }
      // 期待される型がPrimitiveType（型エイリアス）の場合は解決を試みる
      else if (expectedType.kind === "PrimitiveType") {
        const aliasedType = this.resolveTypeAliasRecursively(expectedType.name)
        if (aliasedType && aliasedType.kind === "RecordType") {
          expectedRecordType = aliasedType as AST.RecordType
        }
        // IntersectionTypeの場合も処理
        else if (aliasedType && aliasedType.kind === "IntersectionType") {
          expectedRecordType = this.extractRecordFromIntersection(
            aliasedType as AST.IntersectionType
          )
          if (expectedRecordType) {
          }
        }
      }
      // 期待される型がIntersectionType（型エイリアス）の場合も処理
      else if (expectedType.kind === "IntersectionType") {
        expectedRecordType = this.extractRecordFromIntersection(
          expectedType as AST.IntersectionType
        )
        if (expectedRecordType) {
        }
      }

      // 期待されるRecord型が見つかった場合、省略されたMaybe型フィールドを自動補完
      if (expectedRecordType) {
        for (const expectedField of expectedRecordType.fields) {
          // フィールドが省略されている場合
          if (!fieldMap.has(expectedField.name)) {
            // Maybe型かどうかをチェック
            const isMaybe = isMaybeTypeUtil(expectedField.type)
            if (isMaybe) {
              // Nothing値を自動設定
              const nothingConstructor = new AST.ConstructorExpression(
                "Nothing",
                [],
                record.line,
                record.column
              )
              const nothingField = new AST.RecordInitField(
                expectedField.name,
                nothingConstructor,
                record.line,
                record.column
              )

              // RecordExpressionのfieldsに追加
              record.fields.push(nothingField)

              // fieldMapにも追加（型推論用）
              // 期待される型（Maybe<T>）を渡してNothingの型を正しく推論
              const nothingType = this.generateConstraintsForExpression(
                nothingConstructor,
                env,
                expectedField.type
              )
              fieldMap.set(
                expectedField.name,
                new AST.RecordField(
                  expectedField.name,
                  nothingType,
                  record.line,
                  record.column
                )
              )
            }
          }
        }
      }
    }

    // Mapから配列に変換
    const fields = Array.from(fieldMap.values())
    return new AST.RecordType(fields, record.line, record.column)
  }

  private generateConstraintsForRecordAccess(
    access: AST.RecordAccess,
    env: Map<string, AST.Type>
  ): AST.Type {
    const recordType = this.generateConstraintsForExpression(access.record, env)

    // 配列の.lengthアクセスを特別に処理
    if (access.fieldName === "length") {
      // recordTypeが配列型であることを確認するための制約を追加
      const elementType = this.freshTypeVariable(access.line, access.column)
      const arrayType = new AST.GenericType(
        "Array",
        [elementType],
        access.line,
        access.column
      )
      this.addConstraint(
        new TypeConstraint(
          recordType,
          arrayType,
          access.line,
          access.column,
          "Array length access"
        )
      )
      // lengthはInt型を返す
      return new AST.PrimitiveType("Int", access.line, access.column)
    }

    // まず、recordTypeがStructTypeかどうかを直接チェック
    if (recordType.kind === "StructType") {
      const structType = recordType as AST.StructType
      const field = structType.fields.find((f) => f.name === access.fieldName)
      if (field) {
        return field.type
      } else {
        this.errors.push(
          new TypeInferenceError(
            `Field '${access.fieldName}' does not exist on struct '${structType.name}'`,
            access.line,
            access.column,
            `Field access .${access.fieldName}`
          )
        )
        return this.freshTypeVariable(access.line, access.column)
      }
    }

    // 型変数やその他の場合は、従来の制約ベースのアプローチを使用
    const fieldType = this.freshTypeVariable(access.line, access.column)

    // 構造的制約を常に作成 - unificationプロセスで解決
    // これにより、StructType と RecordType の両方が適切に処理される
    const expectedRecordType = new AST.RecordType(
      [
        new AST.RecordField(
          access.fieldName,
          fieldType,
          access.line,
          access.column
        ),
      ],
      access.line,
      access.column
    )

    // レコードまたは構造体が指定フィールドと互換性があることを制約
    this.addConstraint(
      new TypeConstraint(
        recordType,
        expectedRecordType,
        access.line,
        access.column,
        `Field access .${access.fieldName}`
      )
    )

    return fieldType
  }

  private generateConstraintsForArrayLiteral(
    arrayLiteral: AST.ArrayLiteral,
    env: Map<string, AST.Type>
  ): AST.Type {
    if (arrayLiteral.elements.length === 0) {
      // 空配列の場合、要素型は新しい型変数
      const elementType = this.freshTypeVariable(
        arrayLiteral.line,
        arrayLiteral.column
      )
      return new AST.GenericType(
        "Array",
        [elementType],
        arrayLiteral.line,
        arrayLiteral.column
      )
    }

    // 最初の要素の型を推論
    const firstElementType = this.generateConstraintsForExpression(
      arrayLiteral.elements[0],
      env
    )

    // すべての要素が同じ型であることを制約として追加
    for (let i = 1; i < arrayLiteral.elements.length; i++) {
      const elementType = this.generateConstraintsForExpression(
        arrayLiteral.elements[i],
        env
      )
      this.addConstraint(
        new TypeConstraint(
          firstElementType,
          elementType,
          arrayLiteral.elements[i].line,
          arrayLiteral.elements[i].column,
          `Array element type consistency`
        )
      )
    }

    return new AST.GenericType(
      "Array",
      [firstElementType],
      arrayLiteral.line,
      arrayLiteral.column
    )
  }

  private generateConstraintsForArrayAccess(
    arrayAccess: AST.ArrayAccess,
    env: Map<string, AST.Type>
  ): AST.Type {
    const arrayType = this.generateConstraintsForExpression(
      arrayAccess.array,
      env
    )
    const indexType = this.generateConstraintsForExpression(
      arrayAccess.index,
      env
    )

    // インデックスはInt型でなければならない
    this.addConstraint(
      new TypeConstraint(
        indexType,
        new AST.PrimitiveType(
          "Int",
          arrayAccess.index.line,
          arrayAccess.index.column
        ),
        arrayAccess.index.line,
        arrayAccess.column,
        "Array index must be Int"
      )
    )

    // 戻り値の型変数を作成
    const resultType = this.freshTypeVariable(
      arrayAccess.line,
      arrayAccess.column
    )

    // ArrayAccessに対する特別な制約を追加
    this.addArrayOrTupleAccessConstraint(arrayType, resultType, arrayAccess)

    return resultType
  }

  private addArrayOrTupleAccessConstraint(
    arrayType: AST.Type,
    resultType: AST.Type,
    arrayAccess: AST.ArrayAccess
  ): void {
    // ArrayAccessConstraintという特別な制約タイプを作成
    // この制約は統一化時に特別に処理される
    const constraint = new ArrayAccessConstraint(
      arrayType,
      resultType,
      arrayAccess.line,
      arrayAccess.column,
      "Array or Tuple access"
    )

    this.constraints.push(constraint)
  }

  private generateConstraintsForListSugar(
    listSugar: AST.ListSugar,
    env: Map<string, AST.Type>
  ): AST.Type {
    if (listSugar.elements.length === 0) {
      // 空リストの場合、要素型は新しい型変数
      const elementType = this.freshTypeVariable(
        listSugar.line,
        listSugar.column
      )
      return new AST.GenericType(
        "List",
        [elementType],
        listSugar.line,
        listSugar.column
      )
    }

    // 最初の要素の型を推論
    const firstElementType = this.generateConstraintsForExpression(
      listSugar.elements[0],
      env
    )

    // すべての要素が同じ型であることを制約として追加
    for (let i = 1; i < listSugar.elements.length; i++) {
      const elementType = this.generateConstraintsForExpression(
        listSugar.elements[i],
        env
      )
      this.addConstraint(
        new TypeConstraint(
          firstElementType,
          elementType,
          listSugar.elements[i].line,
          listSugar.elements[i].column,
          `List element type consistency`
        )
      )
    }

    return new AST.GenericType(
      "List",
      [firstElementType],
      listSugar.line,
      listSugar.column
    )
  }

  private generateConstraintsForConsExpression(
    consExpr: AST.ConsExpression,
    env: Map<string, AST.Type>
  ): AST.Type {
    // head : tail の型推論
    const headType = this.generateConstraintsForExpression(consExpr.left, env)
    const tailType = this.generateConstraintsForExpression(consExpr.right, env)

    // tailはList<T>型でなければならない
    const expectedTailType = new AST.GenericType(
      "List",
      [headType],
      consExpr.right.line,
      consExpr.right.column
    )

    this.addConstraint(
      new TypeConstraint(
        tailType,
        expectedTailType,
        consExpr.right.line,
        consExpr.right.column,
        "Cons tail must be List type"
      )
    )

    // 結果の型もList<T>
    return new AST.GenericType(
      "List",
      [headType],
      consExpr.line,
      consExpr.column
    )
  }

  private generateConstraintsForRangeLiteral(
    range: AST.RangeLiteral,
    env: Map<string, AST.Type>
  ): AST.Type {
    // 範囲リテラルの開始と終了値の型を推論
    const startType = this.generateConstraintsForExpression(range.start, env)
    const endType = this.generateConstraintsForExpression(range.end, env)

    // 開始と終了は同じ型でなければならない
    this.addConstraint(
      new TypeConstraint(
        startType,
        endType,
        range.line,
        range.column,
        "Range start and end must have same type"
      )
    )

    // 範囲は数値型（Int）のリストを返す
    const intType = new AST.PrimitiveType("Int", range.line, range.column)

    this.addConstraint(
      new TypeConstraint(
        startType,
        intType,
        range.start.line,
        range.start.column,
        "Range values must be integers"
      )
    )

    // 範囲リテラルはArray<Int>を返す
    return new AST.GenericType("Array", [intType], range.line, range.column)
  }

  private generateConstraintsForListComprehension(
    comp: AST.ListComprehension,
    env: Map<string, AST.Type>
  ): AST.Type {
    // 内包表記用の環境を作成
    const compEnv = new Map(env)

    // ジェネレータを処理してスコープに変数を追加
    for (const generator of comp.generators) {
      // ジェネレータのiterableの型を推論
      const iterableType = this.generateConstraintsForExpression(
        generator.iterable,
        compEnv
      )

      // iterableはリスト型またはArray型でなければならない
      const elementType = this.freshTypeVariable(
        generator.line,
        generator.column
      )
      const arrayType = new AST.GenericType(
        "Array",
        [elementType],
        generator.line,
        generator.column
      )

      // 配列内包表記では範囲リテラル（Array型）を直接受け入れる
      this.addConstraint(
        new TypeConstraint(
          iterableType,
          arrayType,
          generator.line,
          generator.column,
          `Generator iterable must be Array type for array comprehensions`
        )
      )

      // ジェネレータ変数をスコープに追加
      compEnv.set(generator.variable, elementType)
    }

    // フィルタ条件の型チェック
    for (const filter of comp.filters) {
      const filterType = this.generateConstraintsForExpression(filter, compEnv)
      const boolType = new AST.PrimitiveType("Bool", filter.line, filter.column)

      this.addConstraint(
        new TypeConstraint(
          filterType,
          boolType,
          filter.line,
          filter.column,
          "Array comprehension filter must be Bool"
        )
      )
    }

    // 内包表記の式の型を推論
    const expressionType = this.generateConstraintsForExpression(
      comp.expression,
      compEnv
    )

    // 結果はArray<expressionType>（配列内包表記なのでArrayを返す）
    return new AST.GenericType(
      "Array",
      [expressionType],
      comp.line,
      comp.column
    )
  }

  private generateConstraintsForListComprehensionSugar(
    comp: AST.ListComprehensionSugar,
    env: Map<string, AST.Type>
  ): AST.Type {
    // ListComprehensionSugarは通常のListComprehensionと同じ型推論を行う
    // 内包表記用の環境を作成
    const compEnv = new Map(env)

    // ジェネレータを処理してスコープに変数を追加
    for (const generator of comp.generators) {
      // ジェネレータのiterableの型を推論
      const iterableType = this.generateConstraintsForExpression(
        generator.iterable,
        compEnv
      )

      // iterableはList型またはArray型を受け入れる（バッククォート記法でも配列を受け入れる）
      const elementType = this.freshTypeVariable(
        generator.line,
        generator.column
      )
      const expectedListType = new AST.GenericType(
        "List",
        [elementType],
        generator.line,
        generator.column
      )

      // iterableがListまたはArrayであることを制約として追加
      // 制約解決システムがArray<->List変換を処理する
      this.addConstraint(
        new TypeConstraint(
          iterableType,
          expectedListType,
          generator.line,
          generator.column,
          `Generator iterable must be List type (Array conversion allowed)`
        )
      )

      // ジェネレータ変数をスコープに追加
      compEnv.set(generator.variable, elementType)
    }

    // フィルタ条件の型チェック
    for (const filter of comp.filters) {
      const filterType = this.generateConstraintsForExpression(filter, compEnv)
      const boolType = new AST.PrimitiveType("Bool", filter.line, filter.column)

      this.addConstraint(
        new TypeConstraint(
          filterType,
          boolType,
          filter.line,
          filter.column,
          "List comprehension filter must be Bool"
        )
      )
    }

    // 内包表記の式の型を推論
    const expressionType = this.generateConstraintsForExpression(
      comp.expression,
      compEnv
    )

    // 結果はList<expressionType>
    return new AST.GenericType("List", [expressionType], comp.line, comp.column)
  }

  private generateConstraintsForSpreadExpression(
    spread: AST.SpreadExpression,
    env: Map<string, AST.Type>
  ): AST.Type {
    // スプレッド式自体は中身の式の型と同じ
    return this.generateConstraintsForExpression(spread.expression, env)
  }

  private generateConstraintsForRecordDestructuring(
    recordDestr: AST.RecordDestructuring,
    env: Map<string, AST.Type>
  ): void {
    // 初期化式の型を推論
    const initType = this.generateConstraintsForExpression(
      recordDestr.initializer,
      env
    )

    // パターン内の各フィールドを環境に追加し、適切な型制約を設定
    for (const field of recordDestr.pattern.fields) {
      const variableName = field.alias || field.fieldName
      const fieldType = this.freshTypeVariable(field.line, field.column)

      // フィールド変数を環境に追加
      env.set(variableName, fieldType)
      this.nodeTypeMap.set(field, fieldType)

      // 初期化式がレコード型で、該当フィールドを持つことを制約として追加
      // レコードフィールドアクセスと同等の制約を作成
      const recordFieldType = this.freshTypeVariable(field.line, field.column)
      const expectedRecordType = new AST.RecordType(
        [
          new AST.RecordField(
            field.fieldName,
            recordFieldType,
            field.line,
            field.column
          ),
        ],
        recordDestr.line,
        recordDestr.column
      )

      // 初期化式のレコード型に該当フィールドが存在することを制約として追加
      this.addConstraint(
        new TypeConstraint(
          initType,
          expectedRecordType,
          field.line,
          field.column,
          `Record destructuring field ${field.fieldName}`
        )
      )

      // フィールド変数の型とレコードフィールドの型が一致することを制約として追加
      this.addConstraint(
        new TypeConstraint(
          fieldType,
          recordFieldType,
          field.line,
          field.column,
          `Record destructuring field type ${field.fieldName}`
        )
      )
    }
  }

  private generateConstraintsForStructDestructuring(
    structDestr: AST.StructDestructuring,
    env: Map<string, AST.Type>
  ): void {
    // 初期化式の型を推論
    const initType = this.generateConstraintsForExpression(
      structDestr.initializer,
      env
    )

    // 初期化式が指定された構造体型であることを制約として追加
    const expectedStructType = env.get(structDestr.pattern.structName)
    if (expectedStructType && expectedStructType.kind === "StructType") {
      this.addConstraint(
        new TypeConstraint(
          initType,
          expectedStructType,
          structDestr.line,
          structDestr.column,
          `Struct destructuring of ${structDestr.pattern.structName}`
        )
      )

      // パターン内の各フィールドを環境に追加し、適切な型制約を設定
      const structType = expectedStructType as AST.StructType
      for (const field of structDestr.pattern.fields) {
        const variableName = field.alias || field.fieldName

        // 構造体定義から該当フィールドの型を取得
        const structField = structType.fields.find(
          (f) => f.name === field.fieldName
        )
        if (structField) {
          // フィールドの実際の型を使用
          env.set(variableName, structField.type)
          this.nodeTypeMap.set(field, structField.type)

          // 分割代入された変数を追跡するために、仮想的な変数宣言ノードを作成してnodeTypeMapに追加
          const virtualVarDecl = {
            kind: "VariableDeclaration",
            name: variableName,
            type: structField.type,
            line: field.line,
            column: field.column,
            isDestructured: true,
          }
          this.nodeTypeMap.set(virtualVarDecl, structField.type)
        } else {
          // フィールドが見つからない場合はエラー
          this.errors.push(
            new TypeInferenceError(
              `Field '${field.fieldName}' does not exist in struct ${structDestr.pattern.structName}`,
              field.line,
              field.column
            )
          )
          const fieldType = this.freshTypeVariable(field.line, field.column)
          env.set(variableName, fieldType)
        }
      }
    } else {
      // 構造体型が見つからない場合はfreshTypeVariableで処理
      for (const field of structDestr.pattern.fields) {
        const variableName = field.alias || field.fieldName
        const fieldType = this.freshTypeVariable(field.line, field.column)
        env.set(variableName, fieldType)
      }
    }
  }
}

// Convenience function for type inference
export function infer(
  statements: AST.Statement[],
  filePath?: string
): InferenceResult {
  const inference = new TypeInferenceSystem()
  const program = new AST.Program(statements)
  const result = inference.infer(program, filePath)

  const inferredTypes = new Map<string, AST.Type>()
  const typeEnvironment = new Map<string, AST.Type>()

  extractInferredTypes(statements, result, inferredTypes, typeEnvironment)

  return {
    errors: result.errors,
    inferredTypes,
    typeEnvironment,
  }
}

function extractInferredTypes(
  statements: AST.Statement[],
  result: TypeInferenceSystemResult,
  inferredTypes: Map<string, AST.Type>,
  typeEnvironment: Map<string, AST.Type>
): void {
  for (const stmt of statements) {
    processStatement(stmt, result, inferredTypes, typeEnvironment)
  }
}

function processStatement(
  stmt: AST.Statement,
  result: TypeInferenceSystemResult,
  inferredTypes: Map<string, AST.Type>,
  typeEnvironment: Map<string, AST.Type>
): void {
  if (stmt instanceof AST.VariableDeclaration) {
    processVariableDeclaration(stmt, result, inferredTypes, typeEnvironment)
  } else if (stmt instanceof AST.FunctionDeclaration) {
    processFunctionDeclaration(stmt, result, inferredTypes, typeEnvironment)
  } else if (stmt instanceof AST.StructDeclaration) {
    processStructDeclaration(stmt, result, inferredTypes, typeEnvironment)
  }
}

function processVariableDeclaration(
  stmt: AST.VariableDeclaration,
  result: TypeInferenceSystemResult,
  inferredTypes: Map<string, AST.Type>,
  typeEnvironment: Map<string, AST.Type>
): void {
  addResolvedTypeToMaps(stmt, stmt.name, result, inferredTypes, typeEnvironment)
}

function processFunctionDeclaration(
  stmt: AST.FunctionDeclaration,
  result: TypeInferenceSystemResult,
  inferredTypes: Map<string, AST.Type>,
  typeEnvironment: Map<string, AST.Type>
): void {
  addResolvedTypeToMaps(stmt, stmt.name, result, inferredTypes, typeEnvironment)
}

function processStructDeclaration(
  stmt: AST.StructDeclaration,
  result: TypeInferenceSystemResult,
  inferredTypes: Map<string, AST.Type>,
  typeEnvironment: Map<string, AST.Type>
): void {
  addResolvedTypeToMaps(stmt, stmt.name, result, inferredTypes, typeEnvironment)
}

function addResolvedTypeToMaps(
  stmt: AST.Statement,
  name: string,
  result: TypeInferenceSystemResult,
  inferredTypes: Map<string, AST.Type>,
  typeEnvironment: Map<string, AST.Type>
): void {
  const type = result.nodeTypeMap.get(stmt)
  if (type) {
    const resolvedType = result.substitution.apply(type)
    inferredTypes.set(name, resolvedType)
    typeEnvironment.set(name, resolvedType)
  }
}
// TemplateExpression の型推論メソッドを TypeInferenceSystem クラスに追加
;(
  TypeInferenceSystem.prototype as any
).generateConstraintsForTemplateExpression = function (
  templateExpr: AST.TemplateExpression,
  env: Map<string, AST.Type>
): AST.Type {
  // テンプレートリテラルの結果型は常にString
  const resultType = new AST.PrimitiveType(
    "String",
    templateExpr.line,
    templateExpr.column
  )

  // 各埋め込み式の型を推論し、それらがtoString可能であることを確認
  for (const part of templateExpr.parts) {
    if (typeof part !== "string") {
      // 埋め込み式の型を推論
      const exprType = this.generateConstraintsForExpression(part, env)
      ;(this as any).nodeTypeMap.set(part, exprType)

      // TODO: toString可能な型制約を追加する場合はここで実装
      // 現在は全ての型がtoString可能と仮定
    }
  }

  ;(this as any).nodeTypeMap.set(templateExpr, resultType)
  return resultType
}

// TypeAssertion の型推論メソッドを TypeInferenceSystem クラスに追加
;(TypeInferenceSystem.prototype as any).generateConstraintsForTypeAssertion =
  function (
    assertion: AST.TypeAssertion,
    env: Map<string, AST.Type>
  ): AST.Type {
    // 元の式の型を推論
    const exprType = this.generateConstraintsForExpression(
      assertion.expression,
      env
    )

    // 型アサーションの場合、制約を生成せずに直接ターゲット型を返す
    // これにより型チェックを緩める（TypeScript風の動作）
    const targetType = assertion.targetType

    // ノードタイプマップに記録
    ;(this as any).nodeTypeMap.set(assertion.expression, exprType)
    ;(this as any).nodeTypeMap.set(assertion, targetType)

    return targetType
  }

// Promise関連の型推論メソッドは上記のクラス内に実装済み

// TryExpression の型推論メソッドを TypeInferenceSystem クラスに追加
;(TypeInferenceSystem.prototype as any).generateConstraintsForTryExpression =
  function (tryExpr: AST.TryExpression, env: Map<string, AST.Type>): AST.Type {
    // 内部式の型を推論
    const innerType = this.generateConstraintsForExpression(
      tryExpr.expression,
      env
    )
    ;(this as any).nodeTypeMap.set(tryExpr.expression, innerType)

    // エラー型を決定
    let errorType: AST.Type
    if (tryExpr.errorType) {
      errorType = tryExpr.errorType
    } else {
      // デフォルトはString型
      errorType = new AST.PrimitiveType("String", tryExpr.line, tryExpr.column)
    }

    // Promise型かどうかチェック
    const isPromiseType = isPromiseTypeUtil(innerType)

    if (isPromiseType) {
      // Promise関数型 (Unit -> Promise<T>) -> Unit -> Promise<Either<L, T>>
      let valueType = innerType

      // Function型の場合、戻り値型(Promise<T>)からT部分を取得
      if (innerType.kind === "FunctionType") {
        const funcType = innerType as AST.FunctionType
        const returnType = funcType.returnType
        if (
          returnType.kind === "GenericType" &&
          (returnType as AST.GenericType).name === "Promise" &&
          (returnType as AST.GenericType).typeArguments.length > 0
        ) {
          valueType = (returnType as AST.GenericType).typeArguments[0]
        }
      }
      // 直接Promise<T>の場合、T部分を取得
      else if (
        innerType.kind === "GenericType" &&
        (innerType as AST.GenericType).name === "Promise" &&
        (innerType as AST.GenericType).typeArguments.length > 0
      ) {
        valueType = (innerType as AST.GenericType).typeArguments[0]
      }

      // Either<L, T>を構築
      const eitherType = new AST.GenericType(
        "Either",
        [errorType, valueType],
        tryExpr.line,
        tryExpr.column
      )

      // Promise<Either<L, T>>を構築
      const promiseEitherType = new AST.GenericType(
        "Promise",
        [eitherType],
        tryExpr.line,
        tryExpr.column
      )

      // Unit -> Promise<Either<L, T>>を構築
      const resultType = new AST.FunctionType(
        new AST.PrimitiveType("Unit", tryExpr.line, tryExpr.column),
        promiseEitherType,
        tryExpr.line,
        tryExpr.column
      )

      ;(this as any).nodeTypeMap.set(tryExpr, resultType)
      return resultType
    } else {
      // T -> Unit -> Either<L, T>
      const eitherType = new AST.GenericType(
        "Either",
        [errorType, innerType],
        tryExpr.line,
        tryExpr.column
      )

      // Unit -> Either<L, T>を構築
      const resultType = new AST.FunctionType(
        new AST.PrimitiveType("Unit", tryExpr.line, tryExpr.column),
        eitherType,
        tryExpr.line,
        tryExpr.column
      )

      ;(this as any).nodeTypeMap.set(tryExpr, resultType)
      return resultType
    }
  }

// MethodCall処理のためにTypeInferenceSystemクラスを拡張
