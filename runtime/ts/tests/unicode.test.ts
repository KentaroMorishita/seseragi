import { describe, expect, test } from "bun:test"
import { readFileSync } from "node:fs"
import * as char from "../src/char"
import * as grapheme from "../src/grapheme"
import * as text from "../src/text"
import * as unicode from "../src/unicode"
import { GENERAL_CATEGORY } from "../src/unicode-data"
import { assertUnicodeVersion, UNICODE_VERSION } from "../src/unicode-version"

const ucd = (file: string) =>
  readFileSync(new URL(`../../unicode/ucd/${file}`, import.meta.url), "utf8")
const points = (value: string) =>
  value
    .trim()
    .split(/\s+/u)
    .map((hex) => Number.parseInt(hex, 16))
const scalars = (value: string) => String.fromCodePoint(...points(value))
const content = (line: string) => line.split("#")[0]!.trim()

describe("pinned Unicode conformance", () => {
  test("all official normalization rows and unlisted assigned scalars", () => {
    const forms = [unicode.NFC, unicode.NFD, unicode.NFKC, unicode.NFKD]
    const expectedColumns = [
      [1, 1, 1, 3, 3],
      [2, 2, 2, 4, 4],
      [3, 3, 3, 3, 3],
      [4, 4, 4, 4, 4],
    ]
    const listed = new Set<number>()
    let part = "",
      rows = 0,
      identities = 0
    for (const line of ucd("NormalizationTest.txt").split("\n")) {
      const value = content(line)
      if (!value) continue
      if (value.startsWith("@")) {
        part = value
        continue
      }
      const columns = value.split(";").slice(0, 5).map(scalars)
      if (part === "@Part1") listed.add(columns[0]!.codePointAt(0)!)
      for (let form = 0; form < forms.length; form++) {
        for (let column = 0; column < columns.length; column++) {
          const actual = unicode.normalize(forms[form]!, columns[column]!)
          const expected = columns[expectedColumns[form]![column]!]!
          if (actual !== expected)
            throw new Error(
              `normalization ${forms[form]!.tag}, column ${column + 1}: ${line}`
            )
        }
      }
      rows++
    }
    for (let range = 0; range < GENERAL_CATEGORY.length; range += 3) {
      for (
        let point = GENERAL_CATEGORY[range]!;
        point <= GENERAL_CATEGORY[range + 1]!;
        point++
      ) {
        if (listed.has(point)) continue
        const scalar = String.fromCodePoint(point)
        for (const form of forms) {
          if (unicode.normalize(form, scalar) !== scalar)
            throw new Error(
              `unlisted normalization U+${point.toString(16)} ${form.tag}`
            )
        }
        identities++
      }
    }
    expect(rows).toBeGreaterThan(19000)
    expect(identities).toBeGreaterThan(250000)
  }, 60000)

  test("all official extended grapheme break cases and UTF-8 offsets", () => {
    let rows = 0
    for (const line of ucd("auxiliary/GraphemeBreakTest.txt").split("\n")) {
      const value = content(line)
      if (!value) continue
      const groups = value
        .split("÷")
        .map((group) => group.replaceAll("×", "").trim())
        .filter(Boolean)
        .map(scalars)
      const source = groups.join("")
      const expected = [0]
      for (const group of groups)
        expected.push(expected.at(-1)! + new TextEncoder().encode(group).length)
      expect(grapheme.clusters(source)).toEqual(groups)
      expect(grapheme.length(source)).toBe(groups.length)
      expect(grapheme.byteBoundaries(source)).toEqual(expected)
      rows++
    }
    expect(rows).toBeGreaterThan(700)
  })

  test("all default simple and full case-fold mappings, not Turkic overrides", () => {
    let mappings = 0
    for (const line of ucd("CaseFolding.txt").split("\n")) {
      const value = content(line)
      if (!value) continue
      const [point, status, mapping] = value
        .split(";")
        .map((field) => field.trim())
      if (status === "C" || status === "S")
        expect(unicode.simpleCaseFold(scalars(point!))).toBe(scalars(mapping!))
      if (status === "C" || status === "F")
        expect(unicode.fullCaseFold(scalars(point!))).toBe(scalars(mapping!))
      mappings++
    }
    expect(mappings).toBeGreaterThan(1500)
    expect(unicode.simpleCaseFold("ß")).toBe("ß")
    expect(unicode.fullCaseFold("Straße İ Σς ﬃ")).toBe("strasse i\u0307 σσ ffi")
  })

  test("properties and default special casing are independent of host ICU", () => {
    const oldNormalize = String.prototype.normalize
    const oldLower = String.prototype.toLowerCase
    const oldUpper = String.prototype.toUpperCase
    const forbidden = () => {
      throw new Error("host Unicode operation")
    }
    try {
      String.prototype.normalize = forbidden
      String.prototype.toLowerCase = forbidden
      String.prototype.toUpperCase = forbidden
      expect(unicode.normalize(unicode.NFC, "A👍🏽e\u0301")).toBe("A👍🏽é")
      expect(unicode.normalize(unicode.NFKC, "ﬃ①")).toBe("ffi1")
      expect(text.toLower("ΟΣ ΟΣΑ Σ İ")).toBe("ος οσα σ i\u0307")
      expect(text.toLower("Ο'Σ\u0301! Ο'Σ\u0301Α")).toBe(
        "ο'ς\u0301! ο'σ\u0301α"
      )
      expect(text.toUpper("Straße ﬃ ᾀ")).toBe("STRASSE FFI ἈΙ")
      expect(grapheme.clusters("👩‍💻🇯🇵क्‍क")).toEqual(["👩‍💻", "🇯🇵", "क्‍क"])
      expect(text.caseFold("Straße")).toBe(unicode.fullCaseFold("Straße"))
    } finally {
      String.prototype.normalize = oldNormalize
      String.prototype.toLowerCase = oldLower
      String.prototype.toUpperCase = oldUpper
    }
    expect(unicode.generalCategory("A")).toEqual(unicode.UppercaseLetter)
    expect(unicode.generalCategory("\u{1e6c0}")).toEqual(unicode.OtherLetter)
    expect(unicode.generalCategory("\u{10ffff}")).toEqual(unicode.Unassigned)
    expect(unicode.isAlphabetic("\u0345")).toBe(true)
    expect(unicode.isWhitespace("\u0085")).toBe(true)
    expect(unicode.isWhitespace("\ufeff")).toBe(false)
    expect(unicode.isDecimalDigit("٣")).toBe(true)
    expect(unicode.isDecimalDigit("²")).toBe(false)
    expect(unicode.isMark("\u0301")).toBe(true)
  })

  test("long combining sequences and case contexts do not overflow or scan quadratically", () => {
    const source = `a${"\u0301\u0323".repeat(20000)}`
    const normalized = unicode.normalize(unicode.NFD, source)
    expect(normalized).toBe(
      `a${"\u0323".repeat(20000)}${"\u0301".repeat(20000)}`
    )
    expect(unicode.isNormalized(unicode.NFD, normalized)).toBe(true)
    expect(text.toLower(`A${"Σ\u0301".repeat(20000)}`)).toBe(
      `a${"σ\u0301".repeat(19999)}ς\u0301`
    )
    expect(grapheme.length(source)).toBe(1)
  })

  test("one published version, incompatible runtimes fail closed", () => {
    const manifest = JSON.parse(
      readFileSync(
        new URL("../../unicode/manifest.json", import.meta.url),
        "utf8"
      )
    )
    expect(unicode.version(undefined)).toBe(manifest.version)
    expect(UNICODE_VERSION).toBe(manifest.version)
    expect(() => assertUnicodeVersion(UNICODE_VERSION)).not.toThrow()
    expect(() => assertUnicodeVersion("16.0.0")).toThrow("runtime ABI mismatch")
    expect(() => assertUnicodeVersion("18.0.0")).toThrow("runtime ABI mismatch")
  })
})

describe("text scalar / byte / grapheme boundaries", () => {
  test("distinct units and checked slicing preserve original source", () => {
    const source = "A👍🏽e\u0301"
    expect(text.lengthBytes(source)).toBe(12)
    expect(text.lengthScalars(source)).toBe(5)
    expect(grapheme.length(source)).toBe(3)
    expect(grapheme.byteBoundaries(source)).toEqual([0, 1, 9, 12])
    expect(text.scalarAt(1, source)).toEqual({ tag: "Just", value: "👍" })
    expect(grapheme.at(1, source)).toEqual({ tag: "Just", value: "👍🏽" })
    expect(text.sliceScalars(1, 3, source)).toEqual({
      tag: "Right",
      value: "👍🏽",
    })
    expect(grapheme.slice(1, 3, source)).toEqual({
      tag: "Right",
      value: "👍🏽e\u0301",
    })
    expect(grapheme.slice(3, 3, source)).toEqual({ tag: "Right", value: "" })
    for (const [start, end] of [
      [-1, 2],
      [2, 1],
      [0, 8],
    ]) {
      expect(text.sliceScalars(start!, end!, source)).toEqual({
        tag: "Left",
        value: text.InvalidScalarRange({ start: start!, end: end!, length: 5 }),
      })
      expect(grapheme.slice(start!, end!, source)).toEqual({
        tag: "Left",
        value: grapheme.InvalidGraphemeRange({
          start: start!,
          end: end!,
          length: 3,
        }),
      })
    }
    for (const index of [-1, 9]) {
      expect(text.scalarAt(index, source)).toEqual({ tag: "Nothing" })
      expect(grapheme.at(index, source)).toEqual({ tag: "Nothing" })
    }
    expect(text.isEmpty("")).toBe(true)
    expect(text.isEmpty("\u0000")).toBe(false)
    expect(text.sliceScalars(0, 0, "")).toEqual({ tag: "Right", value: "" })
    expect(grapheme.slice(0, 0, "")).toEqual({ tag: "Right", value: "" })
    expect(grapheme.length("")).toBe(0)
    expect(grapheme.clusters("")).toEqual([])
    expect(grapheme.byteBoundaries("")).toEqual([0])
  })

  test("literal search, scalar-safe empty separators, lines and Unicode whitespace", () => {
    expect(text.concat(["a", "👍"])).toBe("a👍")
    expect(text.join("/", ["a", "👍"])).toBe("a/👍")
    expect(text.split(".", "a..b.")).toEqual(["a", "", "b", ""])
    expect(text.split("", "a👍e\u0301")).toEqual(["a", "👍", "e", "\u0301"])
    expect(text.split("", "")).toEqual([])
    expect(text.split("x", "")).toEqual([""])
    expect(text.replace("aa", "$1", "aaaa")).toBe("$1aa")
    expect(text.replaceAll("aa", "$&", "aaaaa")).toBe("$&$&a")
    expect(text.replaceAll("", "/", "a👍")).toBe("/a/👍/")
    expect(text.replace("", "/", "a👍")).toBe("/a👍")
    expect(text.replaceAll("", "/", "")).toBe("/")
    expect(text.contains("aaaab", `${"a".repeat(100000)}b`)).toBe(true)
    expect(text.contains(`${"a".repeat(5000)}b`, "a".repeat(100000))).toBe(
      false
    )
    expect(text.contains("", "")).toBe(true)
    expect(text.startsWith("👍", "👍🏽")).toBe(true)
    expect(text.endsWith("🏽", "👍🏽")).toBe(true)
    expect(text.lines("\r\na\rb\nc\u0085d\u2028e\u2029")).toEqual([
      "",
      "a",
      "b",
      "c",
      "d",
      "e",
    ])
    expect(text.lines("")).toEqual([])
    expect(text.words(" \u0085hello,\u2003世界!\ufeff ")).toEqual([
      "hello,",
      "世界!\ufeff",
    ])
    expect(text.trim("\u0085 a \u2003")).toBe("a")
    expect(text.trim("\ufeff a \ufeff")).toBe("\ufeff a \ufeff")
    expect(text.trimStart("\u0085 a ")).toBe("a ")
    expect(text.trimEnd(" a \u0085")).toBe(" a")
    expect(text.trim(" \n")).toBe("")
  })

  test("BOM survives UTF-8 roundtrips and detached substrings; Char rejects non-scalars", () => {
    for (const source of [
      "",
      "\ufeff",
      "\ufeffA👍🏽e\u0301",
      "\u0000\u{10ffff}",
    ]) {
      expect(text.decodeUtf8(text.encodeUtf8(source))).toEqual({
        tag: "Right",
        value: source,
      })
      expect(text.decodeUtf8Lossy(text.encodeUtf8(source))).toBe(source)
      expect(text.sliceScalars(0, text.lengthScalars(source), source)).toEqual({
        tag: "Right",
        value: source,
      })
      expect(grapheme.clusters(source).join("")).toBe(source)
    }
    for (const value of [-1, 0xd800, 0xdfff, 0x110000])
      expect(char.fromCodePoint(value)).toEqual({ tag: "Nothing" })
    expect(char.fromCodePoint(0x1f44d)).toEqual({ tag: "Just", value: "👍" })
    expect(char.codePoint("👍")).toBe(0x1f44d)
    expect(char.toString("👍")).toBe("👍")
  })
})
