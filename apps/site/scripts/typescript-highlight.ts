import assert from "node:assert/strict"
import ts from "typescript"

const primitiveTypes = new Set([
  "any",
  "bigint",
  "boolean",
  "never",
  "number",
  "object",
  "string",
  "symbol",
  "undefined",
  "unknown",
  "void",
])

function tokenClass(kind: ts.ClassificationType, text: string): string {
  switch (kind) {
    case ts.ClassificationType.keyword:
      if (text === "true" || text === "false") return "tok-bool"
      return primitiveTypes.has(text) ? "tok-standardType" : "tok-keyword"
    case ts.ClassificationType.comment:
    case ts.ClassificationType.docCommentTagName:
      return "tok-comment"
    case ts.ClassificationType.numericLiteral:
    case ts.ClassificationType.bigintLiteral:
      return "tok-number"
    case ts.ClassificationType.stringLiteral:
    case ts.ClassificationType.regularExpressionLiteral:
      return "tok-string"
    case ts.ClassificationType.operator:
      return "tok-operator"
    case ts.ClassificationType.punctuation:
      return "tok-punctuation"
    case ts.ClassificationType.className:
    case ts.ClassificationType.enumName:
    case ts.ClassificationType.interfaceName:
    case ts.ClassificationType.typeParameterName:
    case ts.ClassificationType.typeAliasName:
      return "tok-typeName"
    case ts.ClassificationType.identifier:
    case ts.ClassificationType.parameterName:
    case ts.ClassificationType.moduleName:
      return "tok-variableName"
    default:
      return ""
  }
}

// Use TypeScript's own context-aware syntactic classifier: a bare scanner
// cannot distinguish regex from division or resume nested template literals.
// This runs only during SSG; no tokenizer or language service ships to readers.
export function highlightTypeScript(source: string) {
  const file = "example.ts"
  const snapshot = ts.ScriptSnapshot.fromString(source)
  const service = ts.createLanguageService({
    getCompilationSettings: () => ({
      noLib: true,
      target: ts.ScriptTarget.Latest,
    }),
    getScriptFileNames: () => [file],
    getScriptVersion: () => "1",
    getScriptSnapshot: (name) => (name === file ? snapshot : undefined),
    getCurrentDirectory: () => "/",
    getDefaultLibFileName: () => "lib.d.ts",
    fileExists: (name) => name === file,
    readFile: (name) => (name === file ? source : undefined),
  })
  try {
    const { spans } = service.getEncodedSyntacticClassifications(file, {
      start: 0,
      length: source.length,
    })
    const parts: Array<{ text: string; className: string }> = []
    let cursor = 0
    for (let index = 0; index < spans.length; index += 3) {
      const [start, length, kind] = spans.slice(index, index + 3)
      assert.ok(
        start >= cursor && length > 0 && start + length <= source.length
      )
      if (start > cursor)
        parts.push({ text: source.slice(cursor, start), className: "" })
      const text = source.slice(start, start + length)
      parts.push({ text, className: tokenClass(kind, text) })
      cursor = start + length
    }
    if (cursor < source.length)
      parts.push({ text: source.slice(cursor), className: "" })
    return parts
  } finally {
    service.dispose()
  }
}
