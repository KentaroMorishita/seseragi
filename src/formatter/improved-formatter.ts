export function formatSeseragiCode(code: string): string {
  const lines = code.split("\n")
  const formatted: string[] = []
  let indentLevel = 0

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]
    const trimmedLine = line.trim()

    const result = processLine(trimmedLine, indentLevel, formatted)
    if (result.skip) continue

    indentLevel = result.indentLevel
    if (result.formattedLine) {
      formatted.push(result.formattedLine)
    }

    indentLevel = updateIndentLevel(trimmedLine, indentLevel)
  }

  return `${formatted.join("\n")}\n`
}

function processLine(
  trimmedLine: string,
  indentLevel: number,
  formatted: string[]
): {
  skip: boolean
  indentLevel: number
  formattedLine?: string
} {
  // 空行の処理
  if (trimmedLine === "") {
    // 連続する空行は1つまで
    if (formatted.length > 0 && formatted[formatted.length - 1] !== "") {
      return { skip: false, indentLevel, formattedLine: "" }
    }
    return { skip: true, indentLevel }
  }

  // コメント行の処理
  if (trimmedLine.startsWith("//")) {
    return { skip: false, indentLevel, formattedLine: trimmedLine }
  }

  // インデントレベルの調整
  if (trimmedLine === "}" || trimmedLine.startsWith("|")) {
    indentLevel = Math.max(0, indentLevel - 1)
  }

  const formattedLine = formatSingleLine(trimmedLine, indentLevel)
  return { skip: false, indentLevel, formattedLine }
}

function updateIndentLevel(trimmedLine: string, indentLevel: number): number {
  if (isMultilineStart(trimmedLine)) {
    return indentLevel + 1
  } else if (trimmedLine.endsWith("{")) {
    return indentLevel + 1
  }
  return indentLevel
}

function formatSingleLine(line: string, indentLevel: number): string {
  // インデント追加
  const indent = "  ".repeat(indentLevel)

  // パイプで始まる行（match caseなど）
  if (line.startsWith("|")) {
    return indent + normalizeSpacing(line)
  }

  // type定義の継続行
  if (line.startsWith("| ") && indentLevel > 0) {
    return indent + normalizeSpacing(line)
  }

  // その他の行
  return indent + normalizeSpacing(line)
}

function normalizeSpacing(line: string): string {
  // 基本的な空白正規化（複数スペースを1つに）
  let result = line.replace(/\s+/g, " ").trim()

  result = normalizeOperators(result)
  result = normalizeBrackets(result)
  result = normalizeLambdas(result)

  return result
}

function normalizeOperators(text: string): string {
  return (
    text
      // 複合演算子を先に処理
      .replace(/\s*>>=\s*/g, " >>= ") // バインド演算子
      .replace(/\s*>>>\s*/g, " >>> ") // フォールドモノイド
      .replace(/\s*::\s*/g, " :: ") // リストコンス
      .replace(/\s*->\s*/g, " -> ") // 矢印
      .replace(/\s*==\s*/g, " == ") // 等価比較
      .replace(/\s*>=\s*/g, " >= ") // 以上
      .replace(/\s*<=\s*/g, " <= ") // 以下

      // 単一文字演算子
      .replace(/\s*:\s*(?![:=])/g, " :") // コロン（型注釈）- 後ろに:や=がない場合のみ
      .replace(/\s*=\s*/g, " = ") // 等号
      .replace(/\s*\|\s*/g, " | ") // パイプ
      .replace(/\s*\+\s*/g, " + ") // プラス
      .replace(/\s*\*\s*/g, " * ") // 掛け算
      .replace(/\s*\/\s*/g, " / ") // 割り算
      .replace(/\s*%\s*/g, " % ") // モジュロ
      .replace(/\s*,\s*/g, ", ") // カンマ

      // マイナスは特別処理（矢印の一部でない場合のみ）
      .replace(/\s+-\s*(?!>)/g, " - ") // マイナス（矢印以外）
      .replace(/(?<!>)\s*-\s*/g, " - ") // 矢印の直後でないマイナス

      // 比較演算子（複合演算子の後に処理）
      .replace(/\s*>\s*(?![>=])/g, " > ") // より大きい（>=, >>でない）
      .replace(/\s*<\s*(?!=)/g, " < ")
  ) // より小さい（<=でない）
}

function normalizeBrackets(text: string): string {
  return text
    .replace(/\(\s+/g, "(") // 開き括弧の後の余分なスペース
    .replace(/\s+\)/g, ")") // 閉じ括弧の前の余分なスペース
    .replace(/\[\s+/g, "[") // 開き角括弧
    .replace(/\s+\]/g, "]") // 閉じ角括弧
    .replace(/\{\s+/g, "{ ") // 開き波括弧
    .replace(/\s+\}/g, " }") // 閉じ波括弧
    .replace(/<\s+/g, "<") // 山括弧
    .replace(/\s+>/g, ">") // 山括弧
}

function normalizeLambdas(text: string): string {
  return text.replace(/\\\s*(\w+)\s*->/g, "\\$1 ->") // \x -> の形式
}

function isMultilineStart(line: string): boolean {
  // 関数定義で等号の後に何もない場合
  if (line.includes("=") && line.trim().endsWith("=")) {
    return true
  }

  // type定義で等号の後に何もない場合
  if (line.startsWith("type ") && line.trim().endsWith("=")) {
    return true
  }

  // match式で改行が必要な場合
  if (line.includes("match ") && !line.includes("|")) {
    return true
  }

  return false
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
  return code
    .split("\n")
    .map((line) => {
      // コメント行はそのまま
      if (line.trim().startsWith("//")) {
        return line
      }
      return normalizeSpacing(line)
    })
    .join("\n")
}
