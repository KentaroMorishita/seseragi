// section-parser.ts - サンプルファイルをセクションごとに分割

export interface CodeSection {
  id: string
  title: string
  description?: string
  code: string
  fileTitle: string
  category: "basics" | "intermediate" | "advanced"
}

export interface ParsedFile {
  title: string
  description: string
  category: "basics" | "intermediate" | "advanced"
  sections: CodeSection[]
}

/**
 * コードファイルをセクションごとに分割
 */
export function parseCodeSections(
  code: string,
  filename: string,
  category: "basics" | "intermediate" | "advanced"
): ParsedFile {
  const lines = code.split("\n")
  const sections: CodeSection[] = []

  // ファイル全体のタイトルと説明を取得
  const fileTitle = extractFileTitle(lines)
  const fileDescription = extractFileDescription(lines)

  // Debug: remove in production
  // console.log(`Parsing file: ${filename}`)
  // console.log(`Total lines: ${lines.length}`)
  // console.log(`File title: ${fileTitle}`)
  // console.log(`File description: ${fileDescription}`)

  let currentSection: {
    title: string
    startLine: number
    endLine?: number
  } | null = null

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i].trim()

    // セクション区切り（// =====...===== 形式）
    // 次の行がタイトル行（// で始まる）の場合のみセクション開始とする
    if (line.startsWith("// ===") && line.endsWith("===") && line.length > 10) {
      const nextLine = lines[i + 1]?.trim()
      if (nextLine?.startsWith("//") && !nextLine.includes("===")) {
        console.log(`Found section marker at line ${i}: "${line}"`)
        // 前のセクションを完了
        if (currentSection) {
          currentSection.endLine = i - 1
          const sectionCode = extractSectionCode(
            lines,
            currentSection.startLine,
            currentSection.endLine
          )
          if (sectionCode.trim()) {
            const section = {
              id: `${filename}-${sections.length + 1}`,
              title: currentSection.title,
              code: sectionCode,
              fileTitle,
              category,
            }
            console.log(
              `Created section: ${section.title} (${section.code.length} chars)`
            )
            sections.push(section)
          }
        }

        // 新しいセクションのタイトルを取得（次の行から）
        const titleLine = lines[i + 1]?.trim()
        if (titleLine?.startsWith("//")) {
          const title = titleLine.replace(/^\/\/\s*/, "").replace(/\s*---$/, "")
          // console.log(`Found section title: "${title}"`)
          currentSection = {
            title,
            startLine: i,
          }
        }
      }
    }
  }

  // 最後のセクションを処理
  if (currentSection) {
    currentSection.endLine = lines.length - 1
    const sectionCode = extractSectionCode(
      lines,
      currentSection.startLine,
      currentSection.endLine
    )
    if (sectionCode.trim()) {
      const section = {
        id: `${filename}-${sections.length + 1}`,
        title: currentSection.title,
        code: sectionCode,
        fileTitle,
        category,
      }
      // console.log(`Created final section: ${section.title} (${section.code.length} chars)`)
      sections.push(section)
    }
  }

  // console.log(`Finished parsing ${filename}: ${sections.length} sections found`)
  // console.log('Section titles:', sections.map(s => s.title))

  return {
    title: fileTitle,
    description: fileDescription,
    category,
    sections,
  }
}

function extractFileTitle(lines: string[]): string {
  for (const line of lines.slice(0, 5)) {
    if (line.trim().startsWith("//") && !line.includes("===")) {
      const title = line.replace(/^\/\/\s*/, "").replace(/\.ssrg$/, "")
      if (title && !title.includes("=")) {
        return title
      }
    }
  }
  return "Seseragiサンプル"
}

function extractFileDescription(lines: string[]): string {
  for (let i = 1; i < Math.min(5, lines.length); i++) {
    const line = lines[i].trim()
    if (
      line.startsWith("//") &&
      !line.includes("===") &&
      !line.includes("---")
    ) {
      return line.replace(/^\/\/\s*/, "")
    }
  }
  return ""
}

function extractSectionCode(
  lines: string[],
  startLine: number,
  endLine: number
): string {
  // セクションタイトル行の後から開始
  // startLine = 最初の === 行
  // startLine + 1 = タイトル行
  // startLine + 2 = 閉じる === 行
  // startLine + 3 = 空行
  // startLine + 4 = コード開始
  const codeStartLine = startLine + 3 // === 行、タイトル行、閉じる=== 行をスキップ

  // 次のセクションまたはファイル末尾まで
  let codeEndLine = endLine

  // 次の === 行を探す（開始マーカーを探す）
  for (let i = startLine + 3; i <= endLine; i++) {
    const line = lines[i]?.trim()
    const nextLine = lines[i + 1]?.trim()
    if (
      line?.startsWith("// ===") &&
      line.endsWith("===") &&
      line.length > 10
    ) {
      if (nextLine?.startsWith("//") && !nextLine.includes("===")) {
        codeEndLine = i - 1
        break
      }
    }
  }

  return lines
    .slice(codeStartLine, codeEndLine + 1)
    .join("\n")
    .trim()
}
