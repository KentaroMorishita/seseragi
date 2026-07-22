import { lintGutter } from "@codemirror/lint"
import { EditorState } from "@codemirror/state"
import {
  EditorView,
  highlightActiveLine,
  highlightActiveLineGutter,
  highlightSpecialChars,
  lineNumbers,
  hoverTooltip,
} from "@codemirror/view"
import type { EditorHover } from "../analysis/hover"
import { editingExtensions } from "./editing-extensions"
import { seseragiLanguage } from "./seseragi-language"
import { seseragiEditorTheme } from "./theme"

export function createEditor(
  parent: HTMLElement,
  source: string,
  onChange: (source: string) => void,
  hoverAt?: (position: number) => EditorHover | undefined
): EditorView {
  return new EditorView({
    parent,
    state: EditorState.create({
      doc: source,
      extensions: [
        lineNumbers(),
        highlightSpecialChars(),
        editingExtensions,
        highlightActiveLine(),
        highlightActiveLineGutter(),
        lintGutter(),
        ...(hoverAt === undefined ? [] : [analysisHoverTooltip(hoverAt)]),
        EditorView.lineWrapping,
        EditorView.contentAttributes.of({
          "aria-label": "Seseragi source editor",
          "aria-keyshortcuts":
            "Control+F Meta+F Control+/ Meta+/ Alt+ArrowUp Alt+ArrowDown",
        }),
        seseragiLanguage,
        ...seseragiEditorTheme,
        EditorView.updateListener.of((update) => {
          if (update.docChanged) onChange(update.state.doc.toString())
        }),
      ],
    }),
  })
}

function analysisHoverTooltip(
  hoverAt: (position: number) => EditorHover | undefined
) {
  return hoverTooltip((_view, position) => {
    const hover = hoverAt(position)
    if (hover === undefined) return null
    return {
      pos: hover.from,
      end: hover.to,
      above: true,
      create: () => {
        const dom = document.createElement("div")
        dom.className = "analysis-hover"
        const signature = document.createElement("code")
        signature.textContent = hover.title
        dom.append(signature)
        if (hover.type !== undefined && !hover.title.endsWith(hover.type)) {
          const type = document.createElement("span")
          type.textContent = `Inferred: ${hover.type}`
          dom.append(type)
        }
        if (hover.remaining.length > 0) {
          const remaining = document.createElement("span")
          remaining.textContent = `Remaining: ${hover.remaining.join(" → ")}`
          dom.append(remaining)
        }
        if (hover.constraints.length > 0) {
          const constraints = document.createElement("span")
          constraints.textContent = `where ${hover.constraints.join(", ")}`
          dom.append(constraints)
        }
        if (hover.module !== undefined) {
          const module = document.createElement("small")
          module.textContent = hover.module
          dom.append(module)
        }
        if (hover.description !== undefined) {
          const description = document.createElement("p")
          description.textContent = hover.description
          dom.append(description)
        }
        return { dom }
      },
    }
  })
}

export function replaceEditorSource(editor: EditorView, source: string): void {
  editor.dispatch({
    changes: { from: 0, to: editor.state.doc.length, insert: source },
  })
}
