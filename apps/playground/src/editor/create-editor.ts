import { lintGutter } from "@codemirror/lint"
import { Compartment, EditorState, StateField } from "@codemirror/state"
import {
  activateHover,
  Decoration,
  type DecorationSet,
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
  WidgetType,
} from "@codemirror/view"
import type { EditorHover } from "../analysis/hover"
import { editingExtensions } from "./editing-extensions"
import { indentationWhitespaceField } from "./indent-whitespace"
import { highlightSeseragi, seseragiLanguage } from "./seseragi-language"
import { seseragiEditorTheme } from "./theme"

const whitespaceCompartment = new Compartment()
const editableCompartment = new Compartment()
const linePinsCompartment = new Compartment()

export function createEditor(
  parent: HTMLElement,
  source: string,
  onChange: (source: string) => void,
  hoverAt?: (position: number) => EditorHover | undefined
): EditorView {
  return new EditorView({
    parent,
    state: createEditorState(source, onChange, hoverAt),
  })
}

export function createEditorState(
  source: string,
  onChange: (source: string) => void,
  hoverAt?: (position: number) => EditorHover | undefined
): EditorState {
  return EditorState.create({
    doc: source,
    extensions: [
      lineNumbers(),
      highlightSpecialChars(),
      editingExtensions,
      highlightActiveLine(),
      highlightActiveLineGutter(),
      lintGutter(),
      whitespaceCompartment.of([]),
      editableCompartment.of(EditorView.editable.of(true)),
      linePinsCompartment.of([]),
      ...(hoverAt === undefined ? [] : analysisTooltipExtensions(hoverAt)),
      EditorView.lineWrapping,
      EditorView.contentAttributes.of({
        "aria-label": "Seseragi source editor",
        "aria-keyshortcuts":
          "F6 Shift+F6 Control+Shift+E Meta+Shift+E Control+F Meta+F Control+H Meta+H Control+/ Meta+/ Alt+ArrowUp Alt+ArrowDown",
      }),
      seseragiLanguage,
      ...seseragiEditorTheme,
      EditorView.updateListener.of((update) => {
        if (update.docChanged) onChange(update.state.doc.toString())
      }),
    ],
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

export function setEditorEditable(editor: EditorView, editable: boolean): void {
  editor.dispatch({
    effects: editableCompartment.reconfigure(EditorView.editable.of(editable)),
  })
  editor.contentDOM.setAttribute("aria-disabled", String(!editable))
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

export type EditorLinePin = Readonly<{ line: number; label: string }>

class LinePinWidget extends WidgetType {
  constructor(private readonly label: string) {
    super()
  }

  override eq(other: LinePinWidget): boolean {
    return other.label === this.label
  }

  override toDOM(): HTMLElement {
    const pin = document.createElement("span")
    pin.className = "cm-line-pin"
    pin.setAttribute("aria-hidden", "true")
    pin.textContent = this.label
    return pin
  }
}

// Tour の Walkthrough の番号（①②）を、エディターの該当する行の先頭にピンとして出す。
// 書き換えでコードがずれても、ピンは同じ行に付いていく。
export function setEditorLinePins(
  editor: EditorView,
  pins: readonly EditorLinePin[]
): void {
  const field = StateField.define<DecorationSet>({
    create: (state) =>
      Decoration.set(
        pins
          .filter(({ line }) => line >= 1 && line <= state.doc.lines)
          .map(({ line, label }) =>
            Decoration.widget({
              widget: new LinePinWidget(label),
              side: -1,
            }).range(state.doc.line(line).from)
          ),
        true
      ),
    update: (decorations, transaction) => decorations.map(transaction.changes),
    provide: (value) => EditorView.decorations.from(value),
  })
  editor.dispatch({
    effects: linePinsCompartment.reconfigure(pins.length === 0 ? [] : field),
  })
}
