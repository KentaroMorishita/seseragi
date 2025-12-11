/**
 * コード生成ユーティリティ関数
 */

/**
 * 演算子をメソッド名に変換
 */
export function operatorToMethodName(op: string): string {
  const opMap: Record<string, string> = {
    "+": "add",
    "-": "sub",
    "*": "mul",
    "/": "div",
    "%": "mod",
    "==": "eq",
    "!=": "ne",
    "<": "lt",
    ">": "gt",
    "<=": "le",
    ">=": "ge",
    "&&": "and",
    "||": "or",
    "!": "not",
  }
  return opMap[op] || op.replace(/[^a-zA-Z0-9]/g, "_")
}

/**
 * 識別子をサニタイズ（アポストロフィを変換）
 * 例: x' -> x_prime, f'' -> f_prime_prime
 */
export function sanitizeIdentifier(name: string): string {
  return name.replace(/'/g, "_prime")
}

/**
 * ビルトインコンストラクタかどうかを判定
 */
export function isBuiltinConstructor(name: string): boolean {
  return ["Just", "Nothing", "Left", "Right", "Empty", "Cons"].includes(name)
}

/**
 * 基本演算子かどうかを判定
 */
export function isBasicOperator(op: string): boolean {
  return [
    "+",
    "-",
    "*",
    "/",
    "%",
    "==",
    "!=",
    "<",
    ">",
    "<=",
    ">=",
    "&&",
    "||",
  ].includes(op)
}

/**
 * 比較演算子かどうかを判定
 */
export function isComparisonOperator(op: string): boolean {
  return ["==", "!=", "<", ">", "<=", ">="].includes(op)
}

/**
 * 論理演算子かどうかを判定
 */
export function isLogicalOperator(op: string): boolean {
  return ["&&", "||"].includes(op)
}

/**
 * 算術演算子かどうかを判定
 */
export function isArithmeticOperator(op: string): boolean {
  return ["+", "-", "*", "/", "%"].includes(op)
}

/**
 * モナド演算子かどうかを判定
 */
export function isMonadOperator(op: string): boolean {
  return ["<$>", "<*>", ">>=", "|>", "<|", "$", "++"].includes(op)
}

/**
 * 文字列をエスケープ
 */
export function escapeString(str: string): string {
  return str
    .replace(/\\/g, "\\\\")
    .replace(/"/g, '\\"')
    .replace(/\n/g, "\\n")
    .replace(/\r/g, "\\r")
    .replace(/\t/g, "\\t")
}

/**
 * JSの予約語リスト
 */
const jsReservedWords = new Set([
  "break",
  "case",
  "catch",
  "continue",
  "debugger",
  "default",
  "delete",
  "do",
  "else",
  "finally",
  "for",
  "function",
  "if",
  "in",
  "instanceof",
  "new",
  "return",
  "switch",
  "this",
  "throw",
  "try",
  "typeof",
  "var",
  "void",
  "while",
  "with",
  "class",
  "const",
  "enum",
  "export",
  "extends",
  "import",
  "super",
  "implements",
  "interface",
  "let",
  "package",
  "private",
  "protected",
  "public",
  "static",
  "yield",
])

/**
 * JS予約語かどうかを判定
 */
export function isJsReservedWord(name: string): boolean {
  return jsReservedWords.has(name)
}

/**
 * 予約語をエスケープした識別子を返す
 */
export function safeIdentifier(name: string): string {
  const sanitized = sanitizeIdentifier(name)
  if (isJsReservedWord(sanitized)) {
    return `_${sanitized}`
  }
  return sanitized
}
