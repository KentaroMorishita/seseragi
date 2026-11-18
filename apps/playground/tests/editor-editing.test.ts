import { describe, expect, test } from "bun:test"
import { insertBracket } from "@codemirror/autocomplete"
import {
  copyLineDown,
  indentLess,
  indentMore,
  moveLineDown,
  redo,
  undo,
} from "@codemirror/commands"
import { foldable } from "@codemirror/language"
import { getSearchQuery, SearchQuery, setSearchQuery } from "@codemirror/search"
import {
  EditorSelection,
  EditorState,
  type StateCommand,
  type Transaction,
} from "@codemirror/state"
import { editorWhitespaceExtensions } from "../src/editor/create-editor"
import {
  editingExtensions,
  editorKeymap,
  editorSelectionClassNames,
  selectionMarkField,
} from "../src/editor/editing-extensions"
import { indentationWhitespaceField } from "../src/editor/indent-whitespace"
import { searchMatchStatus } from "../src/editor/search-panel"
import { seseragiLanguage } from "../src/editor/seseragi-language"
import { editorSelectionColors } from "../src/editor/theme"

const editorExtensions = [seseragiLanguage, editingExtensions]

function runCommand(
  source: string,
  command: StateCommand,
  selection = EditorSelection.cursor(0)
): EditorState {
  let state = EditorState.create({
    doc: source,
    selection,
    extensions: editorExtensions,
  })
  const handled = command({
    state,
    dispatch: (transaction) => {
      state = transaction.state
    },
  })
  expect(handled).toBe(true)
  return state
}

describe("Playground editor operations", () => {
  for (const insert of ["newtext", "xy", ""]) {
    for (const direction of ["forward", "backward"] as const) {
      test(`collapses a fully replaced ${direction} selection after ${JSON.stringify(insert)}`, () => {
        let state = EditorState.create({
          doc: "abcdef",
          selection: EditorSelection.range(
            direction === "forward" ? 1 : 4,
            direction === "forward" ? 4 : 1
          ),
          extensions: [selectionMarkField],
        })
        state = state.update({
          changes: { from: 0, to: 6, insert },
        }).state

        expect(state.doc.toString()).toBe(insert)
        expect(state.selection.main.toJSON()).toEqual({
          anchor: insert.length,
          head: insert.length,
        })
        expect(state.field(selectionMarkField).size).toBe(0)
      })
    }

    test(`keeps multiple replacement cursors and the main range after ${JSON.stringify(insert)}`, () => {
      let state = EditorState.create({
        doc: "abcdef",
        selection: EditorSelection.create(
          [
            EditorSelection.range(0, 1),
            EditorSelection.range(4, 2),
            EditorSelection.range(5, 6),
          ],
          1
        ),
        extensions: editorExtensions,
      })
      state = state.update({ changes: { from: 0, to: 6, insert } }).state

      expect(state.selection.toJSON()).toEqual({
        ranges:
          insert.length === 0
            ? [{ anchor: 0, head: 0 }]
            : [
                { anchor: 0, head: 0 },
                { anchor: insert.length, head: insert.length },
              ],
        main: insert.length === 0 ? 0 : 1,
      })
      expect(state.field(selectionMarkField).size).toBe(0)
    })
  }

  test("preserves valid selections, explicit selections, and surrounding text", () => {
    const state = EditorState.create({
      doc: "abcdef",
      selection: EditorSelection.range(5, 1),
      extensions: editorExtensions,
    })
    const partial = state.update({
      changes: { from: 2, to: 3, insert: "XY" },
    }).state
    expect(partial.doc.toString()).toBe("abXYdef")
    expect(partial.selection.main.toJSON()).toEqual({ anchor: 6, head: 1 })
    expect(partial.field(selectionMarkField).size).toBe(1)

    const explicit = state.update({
      changes: { from: 0, to: 6, insert: "newtext" },
      selection: EditorSelection.single(3, 1),
    }).state
    expect(explicit.selection.main.toJSON()).toEqual({ anchor: 3, head: 1 })
    expect(explicit.field(selectionMarkField).size).toBe(1)

    const surrounding = EditorState.create({
      doc: "before abcdef after",
      selection: EditorSelection.range(8, 11),
      extensions: editorExtensions,
    }).update({ changes: { from: 7, to: 13, insert: "XY" } }).state
    expect(surrounding.doc.toString()).toBe("before XY after")
    expect(surrounding.selection.main.toJSON()).toEqual({ anchor: 9, head: 9 })

    const whole = EditorState.create({
      doc: "abcdef",
      selection: EditorSelection.range(6, 0),
      extensions: editorExtensions,
    }).update({ changes: { from: 0, to: 6, insert: "newtext" } }).state
    expect(whole.selection.main.toJSON()).toEqual({ anchor: 7, head: 0 })
    expect(whole.field(selectionMarkField).size).toBe(1)

    const cursor = EditorState.create({
      doc: "abcdef",
      selection: EditorSelection.cursor(2),
      extensions: editorExtensions,
    }).update({ changes: { from: 0, to: 6, insert: "newtext" } }).state
    expect(cursor.selection.main.toJSON()).toEqual({ anchor: 0, head: 0 })
  })

  test("undo and redo restore the original selection and replacement cursor", () => {
    let state = EditorState.create({
      doc: "abcdef",
      selection: EditorSelection.range(4, 1),
      extensions: editorExtensions,
    })
    state = state.update({
      changes: { from: 0, to: 6, insert: "newtext" },
    }).state
    const dispatch = (transaction: Transaction) => {
      state = transaction.state
    }
    expect(undo({ state, dispatch })).toBe(true)
    expect(state.doc.toString()).toBe("abcdef")
    expect(state.selection.main.toJSON()).toEqual({ anchor: 4, head: 1 })
    expect(state.field(selectionMarkField).size).toBe(1)
    expect(redo({ state, dispatch })).toBe(true)
    expect(state.doc.toString()).toBe("newtext")
    expect(state.selection.main.toJSON()).toEqual({ anchor: 7, head: 7 })
    expect(state.field(selectionMarkField).size).toBe(0)
  })

  test("keeps multiple selections and marks primary and secondary ranges", () => {
    const state = EditorState.create({
      doc: "value + value",
      selection: EditorSelection.create(
        [EditorSelection.range(0, 5), EditorSelection.range(8, 13)],
        1
      ),
      extensions: editorExtensions,
    })

    expect(state.facet(EditorState.allowMultipleSelections)).toBe(true)
    expect(state.selection.ranges).toHaveLength(2)

    const classes: string[] = []
    state
      .field(selectionMarkField)
      .between(0, state.doc.length, (_, __, mark) => {
        classes.push(mark.spec.class)
      })
    expect(classes).toEqual([
      editorSelectionClassNames.secondary,
      editorSelectionClassNames.primary,
    ])
  })

  test("moves and duplicates complete lines", () => {
    const moved = runCommand("first\nsecond\nthird", moveLineDown)
    expect(moved.doc.toString()).toBe("second\nfirst\nthird")

    const duplicated = runCommand("first\nsecond", copyLineDown)
    expect(duplicated.doc.toString()).toBe("first\nfirst\nsecond")
  })

  test("indents and unindents selected lines", () => {
    const selection = EditorSelection.range(0, "first\nsecond".length)
    const indented = runCommand("first\nsecond", indentMore, selection)
    expect(indented.doc.toString()).toBe("  first\n  second")

    let state = indented
    const handled = indentLess({
      state,
      dispatch: (transaction) => {
        state = transaction.state
      },
    })
    expect(handled).toBe(true)
    expect(state.doc.toString()).toBe("first\nsecond")
  })

  test("closes brackets through CodeMirror language data", () => {
    const state = EditorState.create({ extensions: editorExtensions })
    const transaction = insertBracket(state, "(")

    expect(transaction).not.toBeNull()
    expect(transaction?.state.doc.toString()).toBe("()")
    expect(transaction?.state.selection.main.head).toBe(1)
  })

  test("stores search and replacement queries and finds every match", () => {
    let state = EditorState.create({
      doc: "value + value + other",
      extensions: editorExtensions,
    })
    const query = new SearchQuery({ search: "value", replace: "answer" })
    state = state.update({ effects: setSearchQuery.of(query) }).state
    const cursor = query.getCursor(state)
    const matches: Array<{ from: number; to: number }> = []
    for (let match = cursor.next(); !match.done; match = cursor.next()) {
      matches.push({ from: match.value.from, to: match.value.to })
    }

    expect(getSearchQuery(state).replace).toBe("answer")
    expect(matches).toEqual([
      { from: 0, to: 5 },
      { from: 8, to: 13 },
    ])
  })

  test("reports total and current matches for compact search options", () => {
    const state = EditorState.create({
      doc: "value Value values value",
      selection: EditorSelection.range(19, 24),
      extensions: editorExtensions,
    })

    expect(
      searchMatchStatus(
        state,
        new SearchQuery({ search: "value", caseSensitive: true })
      )
    ).toEqual({ current: 3, total: 3, valid: true })
    expect(
      searchMatchStatus(
        state,
        new SearchQuery({ search: "value", wholeWord: true })
      )
    ).toEqual({ current: 3, total: 3, valid: true })
    expect(
      searchMatchStatus(
        state,
        new SearchQuery({ search: "value\\b", regexp: true })
      )
    ).toEqual({ current: 3, total: 3, valid: true })
    expect(
      searchMatchStatus(state, new SearchQuery({ search: "[", regexp: true }))
    ).toEqual({ current: 0, total: 0, valid: false })
    expect(searchMatchStatus(state, new SearchQuery({ search: "" }))).toEqual({
      current: 0,
      total: 0,
      valid: true,
    })
  })

  test("folds indented blocks without declaration-specific rules", () => {
    const source = [
      "fn greeting name: String -> String =",
      '  let prefix = "Hello"',
      "  `$" + "{prefix}, $" + "{name}`",
      'println $ greeting "Seseragi"',
    ].join("\n")
    const state = EditorState.create({
      doc: source,
      extensions: editorExtensions,
    })
    const firstLine = state.doc.line(1)
    const range = foldable(state, firstLine.from, firstLine.to)

    expect(range).toEqual({
      from: firstLine.to,
      to: state.doc.line(3).to,
    })
  })

  test("exposes standard desktop keymaps for every requested operation", () => {
    const keys = new Set(editorKeymap.flatMap(({ key, mac }) => [key, mac]))

    for (const expected of [
      "Mod-f",
      "Mod-h",
      "Mod-d",
      "Mod-/",
      "Alt-ArrowUp",
      "Alt-ArrowDown",
      "Shift-Alt-ArrowUp",
      "Shift-Alt-ArrowDown",
      "Mod-Alt-ArrowUp",
      "Mod-Alt-ArrowDown",
      "Mod-[",
      "Mod-]",
      "Ctrl-Shift-[",
      "Ctrl-Shift-]",
      "Cmd-Alt-[",
      "Cmd-Alt-]",
      "Tab",
    ]) {
      expect(keys.has(expected)).toBe(true)
    }
  })

  test("uses distinct classes for selections and search results", () => {
    expect(new Set(Object.values(editorSelectionClassNames)).size).toBe(
      Object.values(editorSelectionClassNames).length
    )
    expect(new Set(Object.values(editorSelectionColors)).size).toBe(
      Object.values(editorSelectionColors).length
    )
  })

  test("keeps whitespace decorations opt-in and includes trailing spaces", () => {
    expect(editorWhitespaceExtensions(false)).toHaveLength(0)
    expect(editorWhitespaceExtensions(true)).toHaveLength(2)
  })

  test("marks indentation without decorating spaces between tokens", () => {
    const state = EditorState.create({
      doc: "  let value = 1\n\tprintln value",
      extensions: [indentationWhitespaceField],
    })
    const marks: Array<{ from: number; to: number; className: unknown }> = []
    state
      .field(indentationWhitespaceField)
      .between(0, state.doc.length, (from, to, mark) => {
        marks.push({ from, to, className: mark.spec.class })
      })

    expect(marks).toEqual([
      { from: 0, to: 1, className: "ssrg-indent-space" },
      { from: 1, to: 2, className: "ssrg-indent-space" },
      { from: 16, to: 17, className: "cm-highlightTab" },
    ])
  })

  test("uses a quiet marker instead of CodeMirror's full-size space dot", async () => {
    const styles = await Bun.file(
      new URL("../src/styles.css", import.meta.url)
    ).text()
    expect(styles).toContain(".editor-host .ssrg-indent-space")
    expect(styles).toContain("rgb(142 163 154 / 32%) 0 0.7px")
  })

  test("renders primary and secondary selections without underline shadows", async () => {
    const theme = await Bun.file(
      new URL("../src/editor/theme.ts", import.meta.url)
    ).text()
    expect(theme).not.toContain('boxShadow: "inset 0 -2px')
    expect(theme).toContain("editorSelectionColors.primary")
    expect(theme).toContain("editorSelectionColors.secondary")
  })

  test("replaces the default search toolbar with an accessible compact panel", async () => {
    const [panel, styles] = await Promise.all([
      Bun.file(
        new URL("../src/editor/search-panel.ts", import.meta.url)
      ).text(),
      Bun.file(new URL("../src/styles.css", import.meta.url)).text(),
    ])

    for (const accessibleName of [
      "Find and replace",
      "Show replace",
      "Match case",
      "Match whole word",
      "Use regular expression",
      "Previous match",
      "Next match",
      "Replace current match",
      "Replace all matches",
      "Close find and replace",
    ]) {
      expect(panel).toContain(accessibleName)
    }
    expect(panel).toContain('setAttribute("main-field", "true")')
    expect(styles).toContain(".ssrg-search-panel")
    expect(styles).toContain("width: min(560px, calc(100% - 16px))")
    expect(styles).toContain("width: calc(100% - 8px)")
  })
})
