export function formatSeseragiCode(code: string): string {
  // sample_bakと同じような形を維持する保守的なフォーマッター
  const lines = code.split("\n")
  const result: string[] = []

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]
    const trimmed = line.trim()

    // 完全に空の行
    if (trimmed === "") {
      // 連続する空行を制限
      if (result.length > 0 && result[result.length - 1] !== "") {
        result.push("")
      }
      continue
    }

    // コメント行はそのまま保持
    if (trimmed.startsWith("//")) {
      result.push(trimmed)
      continue
    }

    // 他の行は基本的なクリーンアップのみ
    const cleaned = trimmed.replace(/\s+/g, " ")

    // 元のインデントレベルを推測して適用
    const indentLevel = getIndentLevel(cleaned, i, lines)
    const indent = "  ".repeat(indentLevel)

    result.push(indent + cleaned)
  }

  return result.join("\n") + "\n"
}

function getIndentLevel(
  line: string,
  index: number,
  allLines: string[]
): number {
  // パイプで始まる行（match case等）
  if (line.startsWith("|")) {
    return 1
  }

  // 波括弧内の内容
  if (isInsideBraces(index, allLines)) {
    return 1
  }

  // 関数/type定義の継続行
  if (isContinuationLine(index, allLines)) {
    // else文の場合は前の行と同じレベル
    if (line.trim() === "else") {
      return getContinuationIndent(index, allLines)
    }
    return getContinuationIndent(index, allLines) + 1
  }

  // トップレベル
  return 0
}

function isInsideBraces(index: number, lines: string[]): boolean {
  let braceDepth = 0

  for (let i = 0; i <= index; i++) {
    const line = lines[i].trim()
    if (line.includes("{")) braceDepth++
    if (line.includes("}")) braceDepth--
  }

  return braceDepth > 0
}

function isContinuationLine(index: number, lines: string[]): boolean {
  if (index === 0) return false

  // 前の行が等号で終わっているか
  const prevLine = lines[index - 1].trim()
  if (prevLine.endsWith(" =")) {
    return true
  }

  // 前の行がthenで終わっている場合（if-then-else）
  if (prevLine.includes(" then") && !prevLine.includes(" else")) {
    return true
  }

  // match式の継続
  if (prevLine.includes("match ")) {
    return true
  }

  return false
}

function getContinuationIndent(index: number, lines: string[]): number {
  // else文は特別処理 - thenと同じレベル
  const currentLine = lines[index].trim()
  if (currentLine === "else") {
    return 1
  }

  // Right/Leftのような継続行は2レベル
  if (currentLine.startsWith("Right ") || currentLine.startsWith("Left ")) {
    return 2
  }

  return 0
}

export function removeExtraWhitespace(code: string): string {
  return code
    .replace(/[ \t]+/g, " ")
    .replace(/[ \t]+$/gm, "")
    .replace(/\n{3,}/g, "\n\n")
    .replace(/^\n+/, "")
    .replace(/\n+$/, "\n")
}

export function normalizeOperatorSpacing(code: string): string {
  return code
}
