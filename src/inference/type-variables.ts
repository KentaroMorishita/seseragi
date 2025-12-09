/**
 * 型変数クラス (Type Variables) for Seseragi Language
 *
 * 型推論で使用される型変数を定義
 */

import * as AST from "../ast"

// 型変数を表現するクラス
export class TypeVariable extends AST.Type {
  kind = "TypeVariable"
  name: string
  id: number

  constructor(id: number, line: number, column: number) {
    super(line, column)
    this.id = id
    this.name = `t${id}`
  }
}

// 多相型変数を表現するクラス (例: 'a, 'b)
export class PolymorphicTypeVariable extends AST.Type {
  kind = "PolymorphicTypeVariable"
  name: string

  constructor(name: string, line: number, column: number) {
    super(line, column)
    this.name = name
  }
}
