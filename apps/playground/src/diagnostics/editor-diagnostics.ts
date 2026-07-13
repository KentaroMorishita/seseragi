import type { Diagnostic as EditorDiagnostic } from "@codemirror/lint"
import type { Diagnostic } from "../compiler/types"
import { utf8RangeToUtf16 } from "./source-range"

export function toEditorDiagnostics(
  source: string,
  diagnostics: readonly Diagnostic[]
): readonly EditorDiagnostic[] {
  return diagnostics.map((diagnostic) => {
    const range = utf8RangeToUtf16(source, diagnostic.primary)
    return {
      from: range.from,
      to:
        range.to === range.from
          ? Math.min(source.length, range.from + 1)
          : range.to,
      severity:
        diagnostic.severity === "information"
          ? "info"
          : (diagnostic.severity ?? "error"),
      message: `${diagnostic.code}: ${diagnostic.messageKey}`,
    }
  })
}

export function renderDiagnostics(diagnostics: readonly Diagnostic[]): string {
  return diagnostics
    .map(
      (diagnostic) =>
        `${diagnostic.code}: ${diagnostic.messageKey} ` +
        `[${diagnostic.primary.start}, ${diagnostic.primary.end})`
    )
    .join("\n")
}
