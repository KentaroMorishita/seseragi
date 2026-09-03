import { describe, expect, test } from "bun:test"
import { entries } from "../src/map"
import {
  compile,
  compileWith,
  defaultOptions,
  escape as escapeRegex,
  find,
  findAll,
  isMatch,
  replaceAll,
  replaceAllWith,
  split,
} from "../src/regex"

function compiled(pattern: string) {
  const result = compile(pattern)
  if (result.tag === "Left")
    throw new Error(`compile failed at ${result.value.offset}`)
  return result.value
}

describe("portable regex runtime", () => {
  test("reports typed compile failures at UTF-8 byte offsets", () => {
    expect(compile("é[")).toEqual({
      tag: "Left",
      value: { kind: { tag: "UnexpectedRegexEnd" }, offset: 3 },
    })
    expect(compile("(?<名前>a)(?<名前>b)")).toEqual({
      tag: "Left",
      value: {
        kind: { tag: "DuplicateCaptureName", value: "名前" },
        offset: 15,
      },
    })
    expect(compile("(a)\\1")).toEqual({
      tag: "Left",
      value: {
        kind: { tag: "UnsupportedRegexFeature", value: "backreference" },
        offset: 3,
      },
    })
    expect(compile("a+?")).toEqual({
      tag: "Left",
      value: {
        kind: { tag: "UnsupportedRegexFeature", value: "lazy quantifier" },
        offset: 2,
      },
    })
    expect(compile("a{3,2}").tag).toBe("Left")
    expect(compile("\\p{Unknown}").tag).toBe("Left")
  })

  test("uses leftmost-first alternatives and greedy counted quantifiers", () => {
    expect(find(compiled("a|aa"), "xaa")).toMatchObject({
      tag: "Just",
      value: { text: "a", span: { start: 1, end: 2 } },
    })
    expect(find(compiled("a{2,4}"), "xaaaaa")).toMatchObject({
      tag: "Just",
      value: { text: "aaaa", span: { start: 1, end: 5 } },
    })
    expect(find(compiled("(a?)*"), "aa")).toMatchObject({
      tag: "Just",
      value: { text: "aa", span: { start: 0, end: 2 } },
    })
    expect(find(compiled("a{2,}"), "xaaaa")).toMatchObject({
      tag: "Just",
      value: { text: "aaaa", span: { start: 1, end: 5 } },
    })
    expect(find(compiled("(?:ab)?c+"), "zabccc")).toMatchObject({
      tag: "Just",
      value: { text: "abccc", span: { start: 1, end: 6 } },
    })
  })

  test("keeps captures from the last consuming repetition", () => {
    expect(find(compiled("(a?)*"), "a")).toMatchObject({
      tag: "Just",
      value: { captures: [{ tag: "Just", value: { text: "a" } }] },
    })
    expect(find(compiled("(a?)*"), "")).toMatchObject({
      tag: "Just",
      value: { captures: [{ tag: "Nothing" }] },
    })
    expect(find(compiled("(a?)+"), "")).toMatchObject({
      tag: "Just",
      value: { captures: [{ tag: "Just", value: { text: "" } }] },
    })
    expect(find(compiled("(a?){1000000000}"), "")).toMatchObject({
      tag: "Just",
      value: { captures: [{ tag: "Just", value: { text: "" } }] },
    })
    expect(find(compiled("((a)|b)+"), "ab")).toMatchObject({
      tag: "Just",
      value: {
        captures: [{ tag: "Just", value: { text: "b" } }, { tag: "Nothing" }],
      },
    })
  })

  test("returns captures and named captures with UTF-8 byte spans", () => {
    const result = find(compiled("(?<word>\\w+)(?:-(a+))?"), "👍 é́-aa")
    expect(result).toMatchObject({
      tag: "Just",
      value: {
        text: "é́-aa",
        span: { start: 5, end: 12 },
        captures: [
          { tag: "Just", value: { text: "é́", span: { start: 5, end: 9 } } },
          { tag: "Just", value: { text: "aa", span: { start: 10, end: 12 } } },
        ],
      },
    })
    if (result.tag !== "Just") throw new Error("missing match")
    expect(entries(result.value.named)).toEqual([
      [
        "word",
        { tag: "Just", value: { text: "é́", span: { start: 5, end: 9 } } },
      ],
    ])
  })

  test("uses pinned Unicode properties and simple case folding", () => {
    expect(isMatch(compiled("^\\w+$"), "漢́٣_")).toBe(true)
    expect(isMatch(compiled("^\\d+$"), "٣")).toBe(false)
    expect(isMatch(compiled("^\\p{Decimal_Number}+$"), "٣")).toBe(true)
    expect(isMatch(compiled("^\\s$"), "\u0085")).toBe(true)
    expect(isMatch(compiled("^\\s$"), "\ufeff")).toBe(false)
    const options = { ...defaultOptions(undefined), caseInsensitive: true }
    const folded = compileWith(options, "^[A-ZΣ]+$")
    if (folded.tag === "Left") throw new Error("case-fold pattern failed")
    expect(isMatch(folded.value, "aςσ")).toBe(true)
    const complement = compileWith(options, "^\\P{Uppercase_Letter}+$")
    if (complement.tag === "Left")
      throw new Error("case-fold complement pattern failed")
    expect(isMatch(complement.value, "a")).toBe(false)
    expect(isMatch(complement.value, "1")).toBe(true)
  })

  test("supports absolute, multiline and dot-newline options", () => {
    const multiline = compileWith(
      { ...defaultOptions(undefined), multiline: true },
      "^b$"
    )
    if (multiline.tag === "Left") throw new Error("multiline compile failed")
    expect(isMatch(multiline.value, "a\r\nb\n")).toBe(true)
    expect(isMatch(compiled("\\Ab\\z"), "a\nb")).toBe(false)
    expect(isMatch(compiled("."), "\n")).toBe(false)
    const dotAll = compileWith(
      { ...defaultOptions(undefined), dotMatchesNewline: true },
      "."
    )
    if (dotAll.tag === "Left") throw new Error("dot-all compile failed")
    expect(isMatch(dotAll.value, "\n")).toBe(true)
  })

  test("advances one scalar after empty matches", () => {
    expect(findAll(compiled("a*"), "bbb").map((match) => match.span)).toEqual([
      { start: 0, end: 0 },
      { start: 1, end: 1 },
      { start: 2, end: 2 },
      { start: 3, end: 3 },
    ])
    expect(split(compiled(""), "a👍")).toEqual(["", "a", "👍", ""])
    expect(replaceAll(compiled(""), "$1", "a👍")).toBe("$1a$1👍$1")
  })

  test("keeps replacement literal and offers explicit capture replacement", () => {
    const pattern = compiled("(?<digits>[0-9]+)")
    expect(replaceAll(pattern, "$1\\", "a12b3")).toBe("a$1\\b$1\\")
    expect(
      replaceAllWith(
        pattern,
        (match) =>
          `[${match.captures[0]!.tag === "Just" ? match.captures[0]!.value.text : ""}]`,
        "a12b3"
      )
    ).toBe("a[12]b[3]")
  })

  test("escape produces a literal fragment for every metacharacter", () => {
    const literal = `a.^$|?*+()[]{}\\\n👍${String.fromCodePoint(0)}1`
    const pattern = compiled(`^${escapeRegex(literal)}$`)
    expect(isMatch(pattern, literal)).toBe(true)
  })

  test("classifies excluded constructs without invoking a host engine", () => {
    const unsupported = [
      ["(?=a)", "look-around"],
      ["(?<=a)", "look-around"],
      ["(?>a)", "atomic group"],
      ["(?(a)b)", "conditional"],
      ["(?R)", "recursion"],
      ["(?i:a)", "inline flag"],
      ["a++", "possessive quantifier"],
    ] as const
    for (const [pattern, feature] of unsupported) {
      expect(compile(pattern)).toMatchObject({
        tag: "Left",
        value: {
          kind: { tag: "UnsupportedRegexFeature", value: feature },
        },
      })
    }
  })

  test("handles long ambiguous input without host backtracking", () => {
    const pattern = compiled("^(a|aa)*b$")
    expect(isMatch(pattern, `${"a".repeat(40_000)}c`)).toBe(false)
  })
})
