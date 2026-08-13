import type { EditorView } from "@codemirror/view"
import type { FormatWidthPreference } from "../preferences/editor-preferences"

export const minimumAutoFormatWidth = 40
export const maximumAutoFormatWidth = 88

export type EditorWidthMetrics = Readonly<{
  contentWidth: number
  characterWidth: number
  paddingInline: number
}>

export function resolveEditorLineWidth(
  editor: EditorView,
  preference: FormatWidthPreference
): number {
  if (preference.mode === "fixed") return preference.fixed

  const view = editor.contentDOM.ownerDocument.defaultView
  const contentStyle = view?.getComputedStyle(editor.contentDOM)
  const line = editor.contentDOM.querySelector<HTMLElement>(".cm-line")
  const lineStyle = line === null ? undefined : view?.getComputedStyle(line)
  const paddingInline =
    parsePixels(contentStyle?.paddingLeft) +
    parsePixels(contentStyle?.paddingRight) +
    parsePixels(lineStyle?.paddingLeft) +
    parsePixels(lineStyle?.paddingRight)
  return resolveAutoFormatWidth({
    contentWidth: editor.contentDOM.getBoundingClientRect().width,
    characterWidth: editor.defaultCharacterWidth,
    paddingInline,
  })
}

export function resolveAutoFormatWidth(metrics: EditorWidthMetrics): number {
  const available = metrics.contentWidth - metrics.paddingInline
  if (!Number.isFinite(available) || available <= 0) {
    return maximumAutoFormatWidth
  }
  if (!Number.isFinite(metrics.characterWidth) || metrics.characterWidth <= 0) {
    return maximumAutoFormatWidth
  }
  return Math.max(
    minimumAutoFormatWidth,
    Math.min(
      maximumAutoFormatWidth,
      Math.floor(available / metrics.characterWidth)
    )
  )
}

function parsePixels(value: string | undefined): number {
  const parsed = Number.parseFloat(value ?? "0")
  return Number.isFinite(parsed) ? parsed : 0
}
