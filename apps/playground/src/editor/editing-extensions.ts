import { closeBrackets, closeBracketsKeymap } from "@codemirror/autocomplete"
import {
  defaultKeymap,
  history,
  historyKeymap,
  indentWithTab,
} from "@codemirror/commands"
import {
  bracketMatching,
  foldGutter,
  foldKeymap,
  foldService,
  indentOnInput,
} from "@codemirror/language"
import {
  highlightSelectionMatches,
  search,
  searchKeymap,
} from "@codemirror/search"
import {
  EditorState,
  StateField,
  type Extension,
  type Line,
} from "@codemirror/state"
import {
  crosshairCursor,
  Decoration,
  type DecorationSet,
  drawSelection,
  dropCursor,
  EditorView,
  keymap,
  rectangularSelection,
  scrollPastEnd,
  type KeyBinding,
} from "@codemirror/view"

export const editorSelectionClassNames = {
  primary: "cm-primarySelectionText",
  secondary: "cm-secondarySelectionText",
  selectionMatch: "cm-selectionMatch",
  searchMatch: "cm-searchMatch",
  selectedSearchMatch: "cm-searchMatch-selected",
} as const

const primarySelectionMark = Decoration.mark({
  class: editorSelectionClassNames.primary,
})
const secondarySelectionMark = Decoration.mark({
  class: editorSelectionClassNames.secondary,
})

function selectionMarks(selection: EditorState["selection"]): DecorationSet {
  return Decoration.set(
    selection.ranges.flatMap((range, index) => {
      if (range.empty) return []
      const mark =
        index === selection.mainIndex
          ? primarySelectionMark
          : secondarySelectionMark
      return [mark.range(range.from, range.to)]
    }),
    true
  )
}

export const selectionMarkField = StateField.define<DecorationSet>({
  create: (state) => selectionMarks(state.selection),
  update: (marks, transaction) => {
    if (!transaction.docChanged && transaction.selection === undefined) {
      return marks
    }
    return selectionMarks(transaction.newSelection)
  },
  provide: (field) => EditorView.decorations.from(field),
})

function indentationWidth(text: string, tabSize: number): number {
  let width = 0
  for (const character of text) {
    if (character === " ") width += 1
    else if (character === "\t") width += tabSize - (width % tabSize)
    else break
  }
  return width
}

/**
 * StreamLanguage does not produce structural syntax nodes. This generic
 * indentation service keeps folding useful without teaching the editor about
 * Seseragi declarations or other language-specific names.
 */
export const indentationFoldService = foldService.of(
  (state, lineStart, lineEnd) => {
    const line = state.doc.lineAt(lineStart)
    const baseIndent = indentationWidth(line.text, state.tabSize)
    let lastIndentedLine: Line | null = null

    for (let number = line.number + 1; number <= state.doc.lines; number += 1) {
      const candidate = state.doc.line(number)
      if (candidate.text.trim().length === 0) continue

      const candidateIndent = indentationWidth(candidate.text, state.tabSize)
      if (candidateIndent <= baseIndent) {
        if (
          lastIndentedLine &&
          /^[\s]*[}\])]/u.test(candidate.text) &&
          candidate.from > lineEnd
        ) {
          return { from: lineEnd, to: candidate.from }
        }
        break
      }
      lastIndentedLine = candidate
    }

    return lastIndentedLine ? { from: lineEnd, to: lastIndentedLine.to } : null
  }
)

export const editorKeymap: readonly KeyBinding[] = [
  ...closeBracketsKeymap,
  ...searchKeymap,
  ...foldKeymap,
  ...defaultKeymap,
  ...historyKeymap,
  indentWithTab,
]

export const editingExtensions: Extension = [
  EditorState.allowMultipleSelections.of(true),
  history(),
  drawSelection(),
  rectangularSelection(),
  crosshairCursor(),
  dropCursor(),
  scrollPastEnd(),
  closeBrackets(),
  bracketMatching(),
  indentOnInput(),
  foldGutter(),
  indentationFoldService,
  search({ top: true }),
  highlightSelectionMatches({ minSelectionLength: 1 }),
  selectionMarkField,
  keymap.of(editorKeymap),
]
