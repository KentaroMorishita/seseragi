import { describe, expect, test } from "bun:test"
import { toggleBlockComment, toggleComment } from "@codemirror/commands"
import { EditorState, type StateCommand } from "@codemirror/state"
import { classHighlighter, highlightTree } from "@lezer/highlight"
import {
  classifyIdentifier,
  seseragiLanguage,
} from "../src/editor/seseragi-language"

function highlightedTokens(source: string) {
  const tokens: Array<{ text: string; classes: string }> = []
  highlightTree(
    seseragiLanguage.parser.parse(source),
    classHighlighter,
    (from, to, classes) => {
      tokens.push({ text: source.slice(from, to), classes })
    }
  )
  return tokens
}

function runCommand(
  source: string,
  command: StateCommand,
  selection = { anchor: 0, head: 0 }
): string {
  let state = EditorState.create({
    doc: source,
    selection,
    extensions: [seseragiLanguage],
  })
  const handled = command({
    state,
    dispatch: (transaction) => {
      state = transaction.state
    },
  })
  expect(handled).toBe(true)
  return state.doc.toString()
}

describe("Seseragi syntax classification", () => {
  test("classifies language declarations and Effect keywords", () => {
    for (const keyword of [
      "pub",
      "type",
      "fn",
      "effect",
      "do",
      "with",
      "fails",
    ]) {
      expect(classifyIdentifier(keyword)).toBe("keyword")
    }
  })

  test("distinguishes constructors, builtins, booleans, and values", () => {
    expect(classifyIdentifier("Effect")).toBe("builtin-type")
    expect(classifyIdentifier("Player1Wins")).toBe("type-name")
    expect(classifyIdentifier("True")).toBe("bool")
    expect(classifyIdentifier("decide")).toBe("variable")
  })

  test("highlights List literals without swallowing them as templates", () => {
    expect(highlightedTokens("`[1, value + 2]")).toEqual([
      { text: "`[", classes: "tok-punctuation" },
      { text: "1", classes: "tok-number" },
      { text: ",", classes: "tok-punctuation" },
      { text: "value", classes: "tok-variableName" },
      { text: "+", classes: "tok-keyword" },
      { text: "2", classes: "tok-number" },
      { text: "]", classes: "tok-punctuation" },
    ])
  })

  test("highlights record spread as one operator token", () => {
    expect(highlightedTokens("{ ...base, name: value }")).toContainEqual({
      text: "...",
      classes: "tok-keyword",
    })
  })

  test("highlights template interpolation as Seseragi expressions", () => {
    expect(highlightedTokens("`hello ${name'}: ${Badge}`")).toEqual([
      { text: "`hello ", classes: "tok-string" },
      { text: "${", classes: "tok-punctuation" },
      { text: "name'", classes: "tok-variableName" },
      { text: "}", classes: "tok-punctuation" },
      { text: ": ", classes: "tok-string" },
      { text: "${", classes: "tok-punctuation" },
      { text: "Badge", classes: "tok-typeName" },
      { text: "}", classes: "tok-punctuation" },
      { text: "`", classes: "tok-string" },
    ])
  })

  test("keeps escaped markers as text and tracks nested interpolation", () => {
    expect(highlightedTokens("`literal \\${name}`")).toEqual([
      { text: "`literal \\${name}`", classes: "tok-string" },
    ])

    const nested = highlightedTokens("`outer ${render `inner ${name}`}`")
    expect(nested.filter(({ text }) => text === "render" || text === "name"))
      .toEqual([
        { text: "render", classes: "tok-variableName" },
        { text: "name", classes: "tok-variableName" },
      ])
  })

  test("toggles line comments through the CodeMirror comment command", () => {
    const source = "let answer = 42\nprintln answer"
    const commented = runCommand(source, toggleComment, {
      anchor: 0,
      head: source.length,
    })

    expect(commented).toBe("// let answer = 42\n// println answer")
    expect(
      runCommand(commented, toggleComment, {
        anchor: 0,
        head: commented.length,
      })
    ).toBe(source)
  })

  test("toggles block comments through the CodeMirror comment command", () => {
    const source = "value + 1"
    const commented = runCommand(source, toggleBlockComment, {
      anchor: 0,
      head: source.length,
    })

    expect(commented).toBe("/* value + 1 */")
    expect(
      runCommand(commented, toggleBlockComment, {
        anchor: 0,
        head: commented.length,
      })
    ).toBe(source)
  })
})
