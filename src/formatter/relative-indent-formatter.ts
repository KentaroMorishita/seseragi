export function formatSeseragiCode(code: string): string {
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

    // コメント行は元のインデントを保持
    if (trimmed.startsWith("//")) {
      // 元の行からインデントを抽出
      const originalIndent = line.match(/^(\s*)/)?.[1] || ""
      result.push(originalIndent + trimmed)
      continue
    }

    // 基本的なクリーンアップ
    const cleaned = trimmed

    // 相対的なインデントレベルを計算
    const indentLevel = calculateIndentLevel(cleaned, i, lines)
    const indent = "  ".repeat(indentLevel)

    result.push(indent + cleaned)
  }

  return result.join("\n") + "\n"
}

function calculateIndentLevel(
  line: string,
  index: number,
  allLines: string[]
): number {
  // 現在のコンテキストレベルを取得
  const contextLevel = getCurrentContextLevel(index, allLines)

  // トップレベル要素でブロック内にない場合
  if (isTopLevelElement(line) && contextLevel === 0) {
    return 0
  }

  // 要素タイプに応じた相対インデントを追加
  const relativeIndent = getRelativeIndent(line, index, allLines)

  return contextLevel + relativeIndent
}

function isTopLevelElement(line: string): boolean {
  return (
    line.startsWith("fn ") ||
    line.startsWith("let ") ||
    line.startsWith("type ") ||
    line.startsWith("impl ") ||
    line.startsWith("monoid ") ||
    line.startsWith("effectful ")
  )
}

function getCurrentContextLevel(index: number, allLines: string[]): number {
  let level = 0

  // 現在の行の前までのコンテキストを解析
  for (let i = 0; i < index; i++) {
    const line = allLines[i].trim()

    if (line === "" || line.startsWith("//")) continue

    // ブロック開始
    if (line.endsWith("{")) {
      level++
    }

    // ブロック終了
    if (line === "}") {
      level = Math.max(0, level - 1)
    }
  }

  // 現在の行が閉じブラケットの場合、レベルを1つ下げる
  const currentLine = allLines[index].trim()
  if (currentLine === "}") {
    level = Math.max(0, level - 1)
  }

  return level
}

function getRelativeIndent(
  line: string,
  index: number,
  allLines: string[]
): number {
  // パイプで始まる行（match case、type選択肢）
  if (line.startsWith("|")) {
    return 1 // 親から +2スペース
  }

  // else文
  if (line === "else") {
    return 1 // 親から +2スペース
  }

  // Right/Left等の深い継続
  if (line.startsWith("Right ") || line.startsWith("Left ")) {
    return 2 // 親から +4スペース
  }

  // 矢印の後の継続行
  if (isArrowContinuation(index, allLines)) {
    return 2 // 矢印から +4スペース（match caseから +2スペース）
  }

  // 関数本体や式の継続
  if (isExpressionContinuation(index, allLines)) {
    return 1 // 親から +2スペース
  }

  // ブロック内の要素（implブロック内のfnなど）で、トップレベル要素でない場合
  const contextLevel = getCurrentContextLevel(index, allLines)
  if (contextLevel > 0 && !isTopLevelElement(line)) {
    return 1 // 親から +2スペース
  }

  // その他はインデントなし（相対で0）
  return 0
}

function isArrowContinuation(index: number, allLines: string[]): boolean {
  if (index === 0) return false

  const prevLine = allLines[index - 1]?.trim()
  return prevLine && prevLine.endsWith(" ->")
}

function isExpressionContinuation(index: number, allLines: string[]): boolean {
  if (index === 0) return false

  // 直前の行をチェック
  for (let i = index - 1; i >= 0; i--) {
    const prevLine = allLines[i].trim()

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

export function removeExtraWhitespace(code: string): string {
  return code
    .replace(/[ \t]+/g, " ")
    .replace(/[ \t]+$/gm, "")
    .replace(/\n{3,}/g, "\n\n")
    .replace(/^\n+/, "")
    .replace(/\n+$/, "\n")
}

export function normalizeOperatorSpacing(code: string): string {
  // コメント行とその他の行を分けて処理
  const lines = code.split("\n")
  const processedLines = lines.map((line) => {
    const trimmed = line.trim()

    // コメント行はそのまま
    if (trimmed.startsWith("//")) {
      return line
    }

    // 文字列リテラルの部分を一時的に保護
    const stringParts: string[] = []
    let processed = line.replace(/"[^"]*"/g, (match) => {
      const index = stringParts.length
      stringParts.push(match)
      return `__STRING_${index}__`
    })

    // その他の行は演算子のスペーシングを正規化
    processed = processed
      // コロンの処理（型注釈）
      .replace(/(\w+)\s*:\s*/g, "$1 :")
      // 複数文字演算子を最初に処理
      .replace(/\s*>>=\s*/g, " >>= ") // バインド演算子
      .replace(/\s*->\s*/g, " -> ") // 関数矢印
      .replace(/\s*==\s*/g, " == ") // 等価演算子
      .replace(/\s*!=\s*/g, " != ") // 不等価演算子
      .replace(/\s*<=\s*/g, " <= ") // 以下演算子
      .replace(/\s*>=\s*/g, " >= ") // 以上演算子
      .replace(/\s*<\$>\s*/g, " <$> ") // ファンクター演算子
      .replace(/\s*<\*>\s*/g, " <*> ") // アプリカティブ演算子
      // 比較演算子 < > (矢印以外の場合)
      .replace(/(?<![<>=-])\s*<\s*(?![=*$])/g, " < ")
      .replace(/(?<![><=-])\s*>\s*(?![=>])/g, " > ")
      // 代入演算子 =（==, >=, <=, >>=, != でない場合のみ）
      .replace(/(?<![\!<>=>])\s*=\s*(?!=)/g, " = ")
      // 加算演算子 +
      .replace(/(?<=[^\+])\s*\+\s*(?=[^\+=])/g, " + ")
      // 乗算演算子 * (<*> でない場合のみ)
      .replace(/(?<![<])\s*\*\s*(?![>])/g, " * ")
      // 除算演算子 / （コメント // でない場合のみ）
      .replace(/(?<!\/)\s*\/\s*(?!\/)/g, " / ")
      // パイプライン演算子 |
      .replace(/\s*\|\s*/g, " | ")
      // 逆パイプ演算子 ~
      .replace(/\s*~\s*/g, " ~ ")
      // 関数適用演算子 $ (単独の場合のみ)
      .replace(/(?<![<>])\s*\$\s*(?![>])/g, " $ ")

    // 文字列リテラルを復元
    stringParts.forEach((str, index) => {
      processed = processed.replace(`__STRING_${index}__`, str)
    })

    return processed
  })

  return processedLines.join("\n")
}
