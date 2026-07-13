import {
  defaultKeymap,
  history,
  historyKeymap,
  indentWithTab,
} from "@codemirror/commands"
import { bracketMatching, indentOnInput } from "@codemirror/language"
import { lintGutter } from "@codemirror/lint"
import { EditorState } from "@codemirror/state"
import {
  drawSelection,
  EditorView,
  highlightActiveLine,
  highlightActiveLineGutter,
  highlightSpecialChars,
  keymap,
  lineNumbers,
} from "@codemirror/view"
import { seseragiLanguage } from "./seseragi-language"
import { seseragiEditorTheme } from "./theme"

export function createEditor(
  parent: HTMLElement,
  source: string,
  onChange: (source: string) => void
): EditorView {
  return new EditorView({
    parent,
    state: EditorState.create({
      doc: source,
      extensions: [
        lineNumbers(),
        highlightSpecialChars(),
        history(),
        drawSelection(),
        indentOnInput(),
        bracketMatching(),
        highlightActiveLine(),
        highlightActiveLineGutter(),
        lintGutter(),
        EditorView.lineWrapping,
        keymap.of([...defaultKeymap, ...historyKeymap, indentWithTab]),
        seseragiLanguage,
        ...seseragiEditorTheme,
        EditorView.updateListener.of((update) => {
          if (update.docChanged) onChange(update.state.doc.toString())
        }),
      ],
    }),
  })
}

export function replaceEditorSource(editor: EditorView, source: string): void {
  editor.dispatch({
    changes: { from: 0, to: editor.state.doc.length, insert: source },
  })
}
