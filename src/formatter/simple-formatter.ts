export function formatSeseragiCode(code: string): string {
  return (
    code
      .split("\n")
      .map((line) => formatLine(line.trim()))
      .filter((line) => line.length > 0)
      .join("\n") + "\n"
  )
}

function formatLine(line: string): string {
  if (!line || line.startsWith("//")) {
    return line
  }

  // 基本的なスペース正規化
  let formatted = line
    // 複数スペースを1つに
    .replace(/\s+/g, " ")
    // 演算子周りのスペース
    .replace(/\s*->\s*/g, " -> ")
    .replace(/\s*::\s*/g, " :: ")
    .replace(/\s*>>=\s*/g, " >>= ")
    .replace(/\s*>>>\s*/g, " >>> ")
    .replace(/\s*\|\s*/g, " | ")
    .replace(/\s*=\s*/g, " = ")
    .replace(/\s*:\s*(?![:])/g, ": ")
    .replace(/\s*,\s*/g, ", ")
    // 算術演算子
    .replace(/\s*\+\s*/g, " + ")
    .replace(/\s*-\s*(?!>)/g, " - ")
    .replace(/\s*\*\s*/g, " * ")
    .replace(/\s*\/\s*/g, " / ")
    .replace(/\s*%\s*/g, " % ")
    .replace(/\s*==\s*/g, " == ")
    .replace(/\s*>=\s*/g, " >= ")
    .replace(/\s*<=\s*/g, " <= ")
    // 括弧内の不要スペース除去
    .replace(/\(\s+/g, "(")
    .replace(/\s+\)/g, ")")
    .replace(/\[\s+/g, "[")
    .replace(/\s+\]/g, "]")
    .replace(/\{\s+/g, "{ ")
    .replace(/\s+\}/g, " }")

  return formatted
}

export function removeExtraWhitespace(code: string): string {
  return (
    code
      // 複数の連続する空白を1つに
      .replace(/[ \t]+/g, " ")
      // 行末の空白を除去
      .replace(/[ \t]+$/gm, "")
      // 複数の連続する改行を最大2つに
      .replace(/\n{3,}/g, "\n\n")
      // ファイル先頭・末尾の余分な改行を除去
      .replace(/^\n+/, "")
      .replace(/\n+$/, "\n")
  )
}

export function normalizeOperatorSpacing(code: string): string {
  return (
    code
      // 矢印演算子
      .replace(/\s*->\s*/g, " -> ")
      // 二重コロン
      .replace(/\s*::\s*/g, " :: ")
      // バインド演算子
      .replace(/\s*>>=\s*/g, " >>= ")
      // フォールドモノイド
      .replace(/\s*>>>\s*/g, " >>> ")
      // パイプ
      .replace(/\s*\|\s*/g, " | ")
      // 等号
      .replace(/\s*=\s*/g, " = ")
      // コロン（型注釈）
      .replace(/\s*:\s*(?![:])/g, ": ")
      // カンマ
      .replace(/\s*,\s*/g, ", ")
      // 算術演算子
      .replace(/\s*\+\s*/g, " + ")
      .replace(/\s*-\s*(?!>)/g, " - ")
      .replace(/\s*\*\s*/g, " * ")
      .replace(/\s*\/\s*/g, " / ")
      .replace(/\s*%\s*/g, " % ")
      // 比較演算子
      .replace(/\s*==\s*/g, " == ")
      .replace(/\s*>=\s*/g, " >= ")
      .replace(/\s*<=\s*/g, " <= ")
      // 括弧の調整
      .replace(/\(\s+/g, "(")
      .replace(/\s+\)/g, ")")
      .replace(/\[\s+/g, "[")
      .replace(/\s+\]/g, "]")
      .replace(/\{\s+/g, "{ ")
      .replace(/\s+\}/g, " }")
  )
}
