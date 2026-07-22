import { HighlightStyle, syntaxHighlighting } from "@codemirror/language"
import { EditorView } from "@codemirror/view"
import { tags } from "@lezer/highlight"
import { editorSelectionClassNames } from "./editing-extensions"

const colors = {
  text: "#dce9e4",
  muted: "#687c74",
  accent: "#7ed6ad",
  purple: "#c7a0f7",
  orange: "#f1b37a",
  blue: "#82b9f6",
  green: "#a9d989",
}

export const editorSelectionColors = {
  primary: "#397a65aa",
  secondary: "#385d78aa",
  selectionMatch: "#69582188",
  searchMatch: "#6c3b7788",
  selectedSearchMatch: "#a1556dcc",
} as const

export const seseragiEditorTheme = [
  EditorView.theme(
    {
      "&": {
        height: "100%",
        color: colors.text,
        backgroundColor: "#0d1513",
        fontSize: "16px",
        lineHeight: "var(--cm-line-height, 1.4)",
      },
      ".cm-content": {
        caretColor: colors.accent,
        fontFamily: '"SFMono-Regular", Consolas, "Liberation Mono", monospace',
        lineHeight: "var(--cm-line-height, 1.4)",
        padding: "var(--cm-content-padding, 18px 0 28px)",
      },
      ".cm-line": {
        padding: "0 var(--cm-line-inline-padding, 18px)",
      },
      ".cm-cursor, .cm-dropCursor": { borderLeftColor: colors.accent },
      ".cm-selectionBackground, ::selection": {
        backgroundColor: "#27574899 !important",
      },
      [`& .${editorSelectionClassNames.primary}`]: {
        backgroundColor: editorSelectionColors.primary,
        boxShadow: "inset 0 -2px 0 #9af0c9",
      },
      [`& .${editorSelectionClassNames.secondary}`]: {
        backgroundColor: editorSelectionColors.secondary,
        boxShadow: "inset 0 -2px 0 #91ccf8",
      },
      [`& .${editorSelectionClassNames.selectionMatch}`]: {
        backgroundColor: editorSelectionColors.selectionMatch,
        outline: "1px solid #d7b95499",
      },
      [`& .${editorSelectionClassNames.searchMatch}`]: {
        backgroundColor: editorSelectionColors.searchMatch,
        outline: "1px solid #d8a0e599",
      },
      [`& .${editorSelectionClassNames.searchMatch}.${editorSelectionClassNames.selectedSearchMatch}`]:
        {
          backgroundColor: editorSelectionColors.selectedSearchMatch,
          outline: "1px solid #ffc0d2",
        },
      ".cm-activeLine": { backgroundColor: "#13201d" },
      ".cm-gutters": {
        backgroundColor: "#0b1210",
        color: colors.muted,
        borderRight: "1px solid #1b2a26",
        fontSize: "var(--cm-gutter-font-size, 13px)",
      },
      ".cm-lineNumbers .cm-gutterElement": {
        minWidth: "var(--cm-line-number-min-width, 2.5em)",
        padding: "var(--cm-line-number-padding, 0 8px 0 5px)",
      },
      ".cm-activeLineGutter": {
        backgroundColor: "#16231f",
        color: colors.text,
      },
      ".cm-foldPlaceholder": { backgroundColor: "transparent", border: "none" },
      ".cm-panels": {
        backgroundColor: "#101b18",
        color: colors.text,
      },
      ".cm-search": {
        alignItems: "center",
        display: "flex",
        flexWrap: "wrap",
        gap: "6px",
        padding: "8px 10px",
      },
      ".cm-search input": {
        backgroundColor: "#0b1210",
        border: "1px solid #355149",
        borderRadius: "6px",
        color: colors.text,
        padding: "5px 7px",
      },
      ".cm-search button": {
        backgroundColor: "#1a302a",
        border: "1px solid #355149",
        borderRadius: "6px",
        color: colors.text,
        padding: "4px 8px",
      },
      ".cm-search button:hover": {
        backgroundColor: "#24443a",
      },
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
