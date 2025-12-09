/**
 * 型推論エラークラス (Type Inference Errors) for Seseragi Language
 *
 * 型推論で発生するエラーを定義
 */

// 型推論エラークラス
export class TypeInferenceError {
  constructor(
    public message: string,
    public line: number,
    public column: number,
    public context?: string
  ) {}

  toString(): string {
    let result = `Type inference error at line ${this.line}, column ${this.column}: ${this.message}`
    if (this.context) {
      result += `\n  Context: ${this.context}`
    }
    return result
  }
}
