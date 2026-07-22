import { lintGutter } from "@codemirror/lint"
import { EditorState } from "@codemirror/state"
import {
  EditorView,
  highlightActiveLine,
  highlightActiveLineGutter,
  highlightSpecialChars,
  lineNumbers,
} from "@codemirror/view"
import { editingExtensions } from "./editing-extensions"
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
        editingExtensions,
        highlightActiveLine(),
        highlightActiveLineGutter(),
        lintGutter(),
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

export function replaceEditorSource(editor: EditorView, source: string): void {
  editor.dispatch({
    changes: { from: 0, to: editor.state.doc.length, insert: source },
  })
}
