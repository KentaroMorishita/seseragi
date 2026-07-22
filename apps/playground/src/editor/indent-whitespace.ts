import { StateField } from "@codemirror/state"
import { Decoration, type DecorationSet, EditorView } from "@codemirror/view"

const space = Decoration.mark({ class: "cm-highlightSpace" })
const tab = Decoration.mark({ class: "cm-highlightTab" })

function indentationMarks(view: EditorView["state"]): DecorationSet {
  const marks = []
  for (let lineNumber = 1; lineNumber <= view.doc.lines; lineNumber += 1) {
    const line = view.doc.line(lineNumber)
    for (let offset = 0; offset < line.text.length; offset += 1) {
      const character = line.text[offset]
      if (character !== " " && character !== "\t") break
      marks.push(
        (character === " " ? space : tab).range(
          line.from + offset,
          line.from + offset + 1
        )
      )
    }
  }
  return Decoration.set(marks, true)
}

export const indentationWhitespaceField = StateField.define<DecorationSet>({
  create: indentationMarks,
  update(value, transaction) {
    return transaction.docChanged ? indentationMarks(transaction.state) : value
  },
  provide: (field) => EditorView.decorations.from(field),
})
