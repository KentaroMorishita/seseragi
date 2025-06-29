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
  // レコード閉じ括弧の直後でトップレベル要素の場合
  if (isTopLevelElement(line) && isAfterRecordClose(index, allLines)) {
    return 0  // トップレベルに戻る
  }

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

function isAfterRecordClose(index: number, allLines: string[]): boolean {
  if (index === 0) return false
  
  // 前の非空行を探す
  for (let i = index - 1; i >= 0; i--) {
    const prevLine = allLines[i].trim()
    if (prevLine === "" || prevLine.startsWith("//")) continue
    
    // 単独の閉じ括弧の後
    if (prevLine === "}") {
      // その閉じ括弧がレコードの終了かチェック
      let openBraces = 0
      for (let j = 0; j <= i; j++) {
        const line = allLines[j].trim()
        if (line === "" || line.startsWith("//")) continue
        
        for (const char of line) {
          if (char === '{') {
            openBraces++
          } else if (char === '}') {
            openBraces = Math.max(0, openBraces - 1)
          }
        }
      }
      
      // 閉じ括弧で括弧が全て閉じた場合
      return openBraces === 0
    }
    
    // 閉じ括弧以外の行があったら、閉じ括弧直後ではない
    return false
  }
  
  return false
}

function isTopLevelElement(line: string): boolean {
  return (
    line.startsWith("fn ") ||
    line.startsWith("let ") ||
    line.startsWith("type ") ||
    line.startsWith("impl ") ||
    line.startsWith("monoid ") ||
    line.startsWith("effectful ") ||
    line.startsWith("show ")  // show文もトップレベル要素として扱う
  )
}

function getCurrentContextLevel(index: number, allLines: string[]): number {
  let level = 0

  // 現在の行の前までのコンテキストを解析
  for (let i = 0; i < index; i++) {
    const line = allLines[i].trim()

    if (line === "" || line.startsWith("//")) continue

    // 文字列リテラルを除外して括弧をカウント
    const braceChange = countBracesInLine(line)
    level = Math.max(0, level + braceChange)
  }

  // 現在の行が閉じブラケットの場合、レベルを1つ下げる
  const currentLine = allLines[index].trim()
  if (currentLine === "}" || currentLine === "},") {
    level = Math.max(0, level - 1)
  }

  return level
}

function countBracesInLine(line: string): number {
  let braceCount = 0
  let inString = false
  let escapeNext = false

  for (let i = 0; i < line.length; i++) {
    const char = line[i]
    
    if (escapeNext) {
      escapeNext = false
      continue
    }
    
    if (char === '\\') {
      escapeNext = true
      continue
    }
    
    if (char === '"') {
      inString = !inString
      continue
    }
    
    if (!inString) {
      if (char === '{') {
        braceCount++
      } else if (char === '}') {
        braceCount--
      }
    }
  }

  return braceCount
}

function getRelativeIndent(
  line: string,
  index: number,
  allLines: string[]
): number {
  // 閉じ括弧は相対インデントなし
  if (line.trim() === "}") {
    return 0
  }

  // 新しい実装では基本的に相対インデントは0
  // getCurrentContextLevel が既に正しいインデントレベルを返す
  
  // モナド演算子で始まる行（>>=, <*>, <$>）
  if (line.startsWith(">>=") || line.startsWith("<*>") || line.startsWith("<$>")) {
    return 1 // 親から +2スペース
  }

  // パイプで始まる行（match case、type選択肢）
  if (line.startsWith("|")) {
    return 1 // 親から +2スペース
  }

  // 三項演算子の継続行（: で始まる行）
  if (isTernaryContinuation(line, index, allLines)) {
    return 1 // 親から +2スペース（関数本体と同じレベル）
  }

  // else文
  if (line === "else") {
    return 1 // 親から +2スペース
  }

  // Right/Left等の深い継続
  if (line.startsWith("Right ") || line.startsWith("Left ")) {
    return 2 // 親から +4スペース
  }

  // Lambda expressions starting with backslash
  if (line.startsWith("\\")) {
    return 1 // 親から +2スペース
  }

  // 矢印の後の継続行
  if (isArrowContinuation(index, allLines)) {
    return 2 // 矢印から +4スペース（match caseから +2スペース）
  }

  // 関数本体や式の継続
  if (isExpressionContinuation(index, allLines)) {
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

// 三項演算子の継続行かどうかを判定
function isTernaryContinuation(
  line: string,
  index: number,
  allLines: string[]
): boolean {
  // 直近の関数定義から三項演算子の文脈にいるかチェック
  let inTernaryContext = false
  let functionStart = -1
  
  // 関数定義の開始を探す
  for (let i = index - 1; i >= 0; i--) {
    const prevLine = allLines[i].trim()
    if (prevLine === "" || prevLine.startsWith("//")) continue
    
    if (prevLine.startsWith("fn ") && prevLine.endsWith(" =")) {
      functionStart = i
      break
    }
    
    // 他のトップレベル要素が見つかったら終了
    if (prevLine.startsWith("let ") || prevLine.startsWith("type ") || 
        prevLine.startsWith("show ")) {
      return false
    }
  }
  
  if (functionStart === -1) return false
  
  // 関数定義以降で ? が見つかるかチェック
  for (let i = functionStart; i < index; i++) {
    const checkLine = allLines[i].trim()
    if (checkLine === "" || checkLine.startsWith("//")) continue
    
    if (checkLine.includes("?")) {
      inTernaryContext = true
      break
    }
  }
  
  // 三項演算子のコンテキストにいて、
  // 現在の行が : を含むか、または三項演算子の最終部分（toString x等）
  if (inTernaryContext) {
    // : を含む行
    if (line.includes(":") && !line.includes("->") && !line.includes("::")) {
      return true
    }
    
    // 前の行が : で終わる場合の最終表現
    if (index > 0) {
      const prevLine = allLines[index - 1].trim()
      if (prevLine.endsWith(":")) {
        return true
      }
    }
  }
  
  return false
}
