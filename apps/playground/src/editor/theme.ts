import { HighlightStyle, syntaxHighlighting } from "@codemirror/language"
import { EditorView } from "@codemirror/view"
import { tags } from "@lezer/highlight"
import { editorSelectionClassNames } from "./editing-extensions"

const colors = {
  text: "var(--seseragi-syntax-text)",
  muted: "var(--seseragi-syntax-muted)",
  accent: "var(--seseragi-syntax-accent)",
  keyword: "var(--seseragi-syntax-keyword)",
  typeName: "var(--seseragi-syntax-type-name)",
  standardType: "var(--seseragi-syntax-standard-type)",
  operator: "var(--seseragi-syntax-operator)",
  string: "var(--seseragi-syntax-string)",
  punctuation: "var(--seseragi-syntax-punctuation)",
}

export const editorSelectionColors = {
  primary: "var(--cm-selection-primary, #397a65aa)",
  secondary: "var(--cm-selection-secondary, #385d78aa)",
  selectionMatch: "var(--cm-selection-match, #69582188)",
  searchMatch: "var(--cm-search-match, #6c3b7788)",
  selectedSearchMatch: "var(--cm-search-match-selected, #a1556dcc)",
} as const

export const seseragiEditorTheme = [
  EditorView.theme(
    {
      "&": {
        color: colors.text,
        backgroundColor: "var(--cm-background, #0d1513)",
        fontSize: "var(--cm-font-size, 16px)",
        lineHeight: "var(--cm-line-height, 1.4)",
      },
      "&.cm-editor": {
        height: "100%",
      },
      ".cm-content": {
        caretColor: colors.accent,
        fontFamily:
          'var(--cm-font-family, "SFMono-Regular", Consolas, "Liberation Mono", monospace)',
        lineHeight: "var(--cm-line-height, 1.4)",
        padding: "var(--cm-content-padding, 18px 0 28px)",
      },
      ".cm-line": {
        padding: "0 var(--cm-line-inline-padding, 18px)",
      },
      ".cm-cursor, .cm-dropCursor": { borderLeftColor: colors.accent },
      ".cm-selectionBackground, ::selection": {
        backgroundColor: "var(--cm-selection, #27574899) !important",
      },
      [`& .${editorSelectionClassNames.primary}`]: {
        backgroundColor: editorSelectionColors.primary,
      },
      [`& .${editorSelectionClassNames.secondary}`]: {
        backgroundColor: editorSelectionColors.secondary,
      },
      [`& .${editorSelectionClassNames.selectionMatch}`]: {
        backgroundColor: editorSelectionColors.selectionMatch,
        outline: "1px solid var(--cm-selection-match-outline, #d7b95499)",
      },
      [`& .${editorSelectionClassNames.searchMatch}`]: {
        backgroundColor: editorSelectionColors.searchMatch,
        outline: "1px solid var(--cm-search-match-outline, #d8a0e599)",
      },
      [`& .${editorSelectionClassNames.searchMatch}.${editorSelectionClassNames.selectedSearchMatch}`]:
        {
          backgroundColor: editorSelectionColors.selectedSearchMatch,
          outline: "1px solid var(--cm-search-match-selected-outline, #ffc0d2)",
        },
      ".cm-activeLine": { backgroundColor: "var(--cm-active-line, #13201d)" },
      ".cm-gutters": {
        backgroundColor: "var(--cm-gutter-background, #0b1210)",
        color: colors.muted,
        borderRight: "1px solid var(--cm-gutter-border, #1b2a26)",
        fontSize: "var(--cm-gutter-font-size, 13px)",
      },
      ".cm-lineNumbers .cm-gutterElement": {
        minWidth: "var(--cm-line-number-min-width, 2.5em)",
        padding: "var(--cm-line-number-padding, 0 8px 0 5px)",
      },
      ".cm-activeLineGutter": {
        backgroundColor: "var(--cm-active-gutter, #16231f)",
        color: colors.text,
      },
      ".cm-foldPlaceholder": { backgroundColor: "transparent", border: "none" },
      ".cm-panels": {
        color: colors.text,
      },
      ".cm-panels:not(.cm-panels-top)": {
        backgroundColor: "var(--cm-panel-background, #101b18)",
        color: colors.text,
      },
      ".cm-diagnostic": { padding: "6px 10px" },
      ".cm-tooltip": {
        backgroundColor: "var(--cm-tooltip-background, #14211e)",
        border: "1px solid var(--cm-tooltip-border, #294139)",
        color: colors.text,
      },
    },
    { dark: true }
  ),
  syntaxHighlighting(
    HighlightStyle.define([
      { tag: tags.keyword, color: colors.keyword, fontWeight: "600" },
      { tag: tags.typeName, color: colors.typeName },
      { tag: tags.standard(tags.typeName), color: colors.standardType },
      { tag: tags.variableName, color: colors.text },
      { tag: tags.bool, color: colors.typeName },
      { tag: tags.number, color: colors.typeName },
      { tag: tags.string, color: colors.string },
      { tag: tags.comment, color: colors.muted, fontStyle: "italic" },
      { tag: tags.operatorKeyword, color: colors.operator },
      { tag: tags.punctuation, color: colors.punctuation },
    ])
  ),
]
