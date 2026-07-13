import { HighlightStyle, syntaxHighlighting } from "@codemirror/language"
import { EditorView } from "@codemirror/view"
import { tags } from "@lezer/highlight"

const colors = {
  text: "#dce9e4",
  muted: "#687c74",
  accent: "#7ed6ad",
  purple: "#c7a0f7",
  orange: "#f1b37a",
  blue: "#82b9f6",
  green: "#a9d989",
}

export const seseragiEditorTheme = [
  EditorView.theme(
    {
      "&": {
        height: "100%",
        color: colors.text,
        backgroundColor: "#0d1513",
        fontSize: "16px",
      },
      ".cm-content": {
        caretColor: colors.accent,
        fontFamily: '"SFMono-Regular", Consolas, "Liberation Mono", monospace',
        padding: "18px 0 28px",
      },
      ".cm-line": { padding: "0 18px" },
      ".cm-cursor, .cm-dropCursor": { borderLeftColor: colors.accent },
      ".cm-selectionBackground, ::selection": {
        backgroundColor: "#27574888 !important",
      },
      ".cm-activeLine": { backgroundColor: "#13201d" },
      ".cm-gutters": {
        backgroundColor: "#0b1210",
        color: colors.muted,
        borderRight: "1px solid #1b2a26",
      },
      ".cm-activeLineGutter": {
        backgroundColor: "#16231f",
        color: colors.text,
      },
      ".cm-foldPlaceholder": { backgroundColor: "transparent", border: "none" },
      ".cm-diagnostic": { padding: "6px 10px" },
      ".cm-tooltip": {
        backgroundColor: "#14211e",
        border: "1px solid #294139",
        color: colors.text,
      },
    },
    { dark: true }
  ),
  syntaxHighlighting(
    HighlightStyle.define([
      { tag: tags.keyword, color: colors.purple, fontWeight: "600" },
      { tag: tags.typeName, color: colors.orange },
      { tag: tags.standard(tags.typeName), color: "#ffd27d" },
      { tag: tags.variableName, color: colors.text },
      { tag: tags.bool, color: colors.orange },
      { tag: tags.number, color: colors.orange },
      { tag: tags.string, color: colors.green },
      { tag: tags.comment, color: colors.muted, fontStyle: "italic" },
      { tag: tags.operatorKeyword, color: colors.blue },
      { tag: tags.punctuation, color: "#9fb4ac" },
    ])
  ),
]
