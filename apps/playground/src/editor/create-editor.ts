import { lintGutter } from "@codemirror/lint"
import { Compartment, EditorState } from "@codemirror/state"
import {
  activateHover,
  EditorView,
  highlightActiveLine,
  highlightActiveLineGutter,
  highlightSpecialChars,
  highlightTrailingWhitespace,
  hoverTooltip,
  lineNumbers,
  repositionTooltips,
  tooltips,
  ViewPlugin,
} from "@codemirror/view"
import type { EditorHover } from "../analysis/hover"
import { editingExtensions } from "./editing-extensions"
import { indentationWhitespaceField } from "./indent-whitespace"
import { highlightSeseragi, seseragiLanguage } from "./seseragi-language"
import { seseragiEditorTheme } from "./theme"

const whitespaceCompartment = new Compartment()

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
        whitespaceCompartment.of([]),
        ...(hoverAt === undefined ? [] : analysisTooltipExtensions(hoverAt)),
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

export function editorWhitespaceExtensions(visible: boolean) {
  return visible
    ? [indentationWhitespaceField, highlightTrailingWhitespace()]
    : []
}

export function setEditorWhitespaceVisible(
  editor: EditorView,
  visible: boolean
): void {
  editor.dispatch({
    effects: whitespaceCompartment.reconfigure(
      editorWhitespaceExtensions(visible)
    ),
  })
}

function analysisTooltipExtensions(
  hoverAt: (position: number) => EditorHover | undefined
) {
  const tooltip = analysisHoverTooltip(hoverAt)
  return [
    tooltip,
    tooltips({
      parent: document.body,
      tooltipSpace: () => visualViewportSpace(document),
    }),
    EditorView.domEventHandlers({
      pointerup(event, view) {
        if (event.pointerType !== "touch") return false
        activateHover(view, view.state.selection.main.head, 1, {
          tooltip,
          until: (transaction) =>
            transaction.docChanged || transaction.selection !== undefined,
        })
        return false
      },
    }),
    ViewPlugin.fromClass(
      class {
        readonly viewport = window.visualViewport
        readonly reposition = (): void => repositionTooltips(this.view)

        constructor(readonly view: EditorView) {
          this.viewport?.addEventListener("resize", this.reposition)
          this.viewport?.addEventListener("scroll", this.reposition)
        }

        destroy(): void {
          this.viewport?.removeEventListener("resize", this.reposition)
          this.viewport?.removeEventListener("scroll", this.reposition)
        }
      }
    ),
  ]
}

export function visualViewportSpace(ownerDocument: Document): {
  readonly left: number
  readonly right: number
  readonly top: number
  readonly bottom: number
} {
  const viewport = ownerDocument.defaultView?.visualViewport
  const margin = 8
  const left = (viewport?.offsetLeft ?? 0) + margin
  const top = (viewport?.offsetTop ?? 0) + margin
  const width = viewport?.width ?? ownerDocument.documentElement.clientWidth
  const height = viewport?.height ?? ownerDocument.documentElement.clientHeight
  return {
    left,
    right: left + Math.max(0, width - margin * 2),
    top,
    bottom: top + Math.max(0, height - margin * 2),
  }
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
        signature.className = "analysis-hover-signature"
        signature.append(
          ...highlightSeseragi(hover.title).map(({ text, classes }) => {
            const span = document.createElement("span")
            span.textContent = text
            if (classes) span.className = classes
            return span
          })
        )
        dom.append(signature)
        if (hover.type !== undefined && !hover.title.endsWith(hover.type)) {
          const type = document.createElement("span")
          type.className = "analysis-hover-detail"
          type.textContent = `Inferred: ${hover.type}`
          dom.append(type)
        }
        if (hover.remaining.length > 0) {
          const remaining = document.createElement("span")
          remaining.className = "analysis-hover-detail"
          remaining.textContent = `Remaining: ${hover.remaining.join(" → ")}`
          dom.append(remaining)
        }
        if (hover.constraints.length > 0) {
          const constraints = document.createElement("span")
          constraints.className = "analysis-hover-detail"
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
