import type { Monaco } from "@monaco-editor/react"

// 簡易版のLSP初期化（フル実装は後で追加）
export function initializeLSP(monaco: Monaco, _editor: any) {
  // 現時点では基本的なフォーマッター機能のみ実装
  monaco.languages.registerDocumentFormattingEditProvider("seseragi", {
    async provideDocumentFormattingEdits(model) {
      try {
        // 簡易的なフォーマット（実際のフォーマッターを後で統合）
        const text = model.getValue()

        // 基本的なインデント調整のみ
        const formatted = formatSeseragiCode(text)

        return [
          {
            range: model.getFullModelRange(),
            text: formatted,
          },
        ]
      } catch (error) {
        console.error("Formatting error:", error)
        return []
      }
    },
  })

  // 簡易的なホバー情報提供
  monaco.languages.registerHoverProvider("seseragi", {
    provideHover(model, position) {
      const word = model.getWordAtPosition(position)
      if (!word) return null

      // 組み込み型の説明
      const builtinTypes: Record<string, string> = {
        Int: "Integer type",
        Float: "Floating point number type",
        String: "String type",
        Bool: "Boolean type (True or False)",
        Maybe: "Optional value type",
        List: "List type",
        Array: "Array type",
      }

      if (builtinTypes[word.word]) {
        return {
          contents: [
            { value: `**${word.word}**` },
            { value: builtinTypes[word.word] },
          ],
        }
      }

      return null
    },
  })
}

// 簡易的なフォーマッター
function formatSeseragiCode(code: string): string {
  // 基本的なインデント調整
  const lines = code.split("\n")
  let indentLevel = 0
  const formattedLines = []

  for (const line of lines) {
    const trimmed = line.trim()

    // インデントレベルの調整
    if (trimmed.startsWith("end") || trimmed.startsWith("|")) {
      indentLevel = Math.max(0, indentLevel - 1)
    }

    // インデントを適用
    if (trimmed) {
      formattedLines.push("  ".repeat(indentLevel) + trimmed)
    } else {
      formattedLines.push("")
    }

    // 次の行のインデントレベルを決定
    if (
      trimmed.endsWith("=") ||
      trimmed.startsWith("match") ||
      trimmed.startsWith("if")
    ) {
      indentLevel++
    }
    if (trimmed.startsWith("|") && !trimmed.startsWith("|>")) {
      indentLevel++
    }
  }

  return formattedLines.join("\n")
}
