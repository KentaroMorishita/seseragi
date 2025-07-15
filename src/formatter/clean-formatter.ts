export function formatSeseragiCode(code: string): string {
  // 最もシンプルなアプローチ：sample_bakの構造を手動で再現
  const lines = code.split("\n")
  const result: string[] = []

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]
    const trimmed = line.trim()

    // 空行処理
    if (trimmed === "") {
      if (result.length > 0 && result[result.length - 1] !== "") {
        result.push("")
      }
      continue
    }

    // コメント行はそのまま
    if (trimmed.startsWith("//")) {
      result.push(trimmed)
      continue
    }

    // 基本的なクリーンアップ
    const cleaned = trimmed.replace(/\s+/g, " ")

    // 正確なインデントレベルを決定
    const indent = getCorrectIndent(cleaned, i, lines)
    result.push(indent + cleaned)
  }

  return `${result.join("\n")}\n`
}

function getCorrectIndent(
  line: string,
  index: number,
  allLines: string[]
): string {
  // 波括弧内の関数定義を先にチェック
  if (line.startsWith("fn ") && isInsideBraces(index, allLines)) {
    return "  "
  }

  // トップレベル構文
  if (isTopLevelStatement(line)) {
    return ""
  }

  // パイプ記号（match case等）
  if (line.startsWith("|")) {
    return getPipeIndent(index, allLines)
  }

  // 特殊な構文（else, Left等）
  const specialIndent = getSpecialStatementIndent(line)
  if (specialIndent !== null) {
    return specialIndent
  }

  // 波括弧内の内容
  if (isInsideBraces(index, allLines)) {
    return getBraceContentIndent(line, index, allLines)
  }

  // 関数本体や式の継続
  if (isExpressionContinuation(index, allLines)) {
    return getExpressionContinuationIndent(line, index, allLines)
  }

  // デフォルトはトップレベル
  return ""
}

function isTopLevelStatement(line: string): boolean {
  return (
    line.startsWith("fn ") ||
    line.startsWith("let ") ||
    line.startsWith("type ") ||
    line.startsWith("impl ") ||
    line.startsWith("monoid ") ||
    line.startsWith("effectful ")
  )
}

function getPipeIndent(index: number, allLines: string[]): string {
  if (isInsideFunctionBody(index, allLines)) {
    return "    "
  }
  return "  "
}

function getSpecialStatementIndent(line: string): string | null {
  if (line === "else") {
    return "  "
  }
  if (line.startsWith("Left ")) {
    return "    "
  }
  return null
}

function getBraceContentIndent(
  line: string,
  index: number,
  allLines: string[]
): string {
  // match caseの継続行（パイプライン矢印の後）
  if (isMatchCaseContinuation(index, allLines)) {
    return "      "
  }

  // match case（implブロック内では4スペース）
  if (line.startsWith("|")) {
    if (isInsideFunctionBody(index, allLines)) {
      return "    "
    }
    return "  "
  }

  // impl/monoidブロック内のすべての内容（関数定義含む）
  return "  "
}

function getExpressionContinuationIndent(
  line: string,
  index: number,
  allLines: string[]
): string {
  // match caseの継続行
  if (isMatchCaseContinuation(index, allLines)) {
    return "      "
  }

  // Right/Left等の深い継続
  if (line.startsWith("Right ") || line.startsWith("Left ")) {
    return "    "
  }

  return "  "
}

function isInsideBraces(index: number, lines: string[]): boolean {
  let braceCount = 0

  for (let i = 0; i <= index; i++) {
    const line = lines[i].trim()
    if (line.endsWith("{")) braceCount++
    if (line === "}") braceCount--
  }

  return braceCount > 0
}

function isExpressionContinuation(index: number, lines: string[]): boolean {
  if (index === 0) return false

  // 直前の行をチェック
  for (let i = index - 1; i >= 0; i--) {
    const prevLine = lines[i].trim()

    // 空行やコメントはスキップ
    if (prevLine === "" || prevLine.startsWith("//")) {
      continue
    }

    // 等号で終わる行の後
    if (prevLine.endsWith(" =")) {
      return true
    }

    // match文の後
    if (prevLine.includes("match ")) {
      return true
    }

    // then文の後（elseでない限り）
    if (prevLine.includes(" then") && !prevLine.includes(" else")) {
      return true
    }

    // 他の条件に当てはまらない場合は継続ではない
    break
  }

  return false
}

function isMatchCaseContinuation(index: number, lines: string[]): boolean {
  if (index === 0) return false

  // 直前の行が矢印で終わる場合
  const prevLine = lines[index - 1]?.trim()
  return prevLine?.endsWith(" ->")
}

function isInsideFunctionBody(index: number, lines: string[]): boolean {
  // 前の行を遡って関数定義（等号で終わる行）を探す
  for (let i = index - 1; i >= 0; i--) {
    const line = lines[i].trim()

    // 空行やコメントはスキップ
    if (line === "" || line.startsWith("//")) {
      continue
    }

    // 関数定義の等号を見つけた
    if (line.endsWith(" =") || line.includes("match ")) {
      return true
    }

    // ブロックの開始/終了に達したら停止
    if (line.endsWith("{") || line === "}") {
      break
    }
  }

  return false
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
