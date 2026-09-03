import type { Unit } from "./effect"
import { arrayEq, boolEq, type Eq, intEq, stringEq } from "./equality"
import { stringHash } from "./hash"
import {
  empty as emptyMap,
  insert as insertMap,
  mapEq,
  type Map as PersistentMap,
} from "./map"
import type { Ord } from "./sequence"
import {
  type Either,
  Equal,
  Greater,
  Just,
  Left,
  Less,
  type Maybe,
  Nothing,
  Right,
} from "./sum"
import { copySubstring, utf8Width } from "./text-core"
import { simpleFoldEquivalents } from "./unicode-case"
import { alphabetic, categoryIndex, whitespace } from "./unicode-properties"

export type RegexCompileErrorKind =
  | Readonly<{ tag: "UnexpectedRegexEnd" }>
  | Readonly<{ tag: "UnexpectedRegexToken"; value: string }>
  | Readonly<{ tag: "InvalidRegexEscape" }>
  | Readonly<{ tag: "InvalidRegexRange" }>
  | Readonly<{ tag: "InvalidRegexQuantifier" }>
  | Readonly<{ tag: "DuplicateCaptureName"; value: string }>
  | Readonly<{ tag: "UnsupportedRegexFeature"; value: string }>

export type RegexCompileError = Readonly<{
  kind: RegexCompileErrorKind
  offset: number
}>

export type RegexOptions = Readonly<{
  caseInsensitive: boolean
  multiline: boolean
  dotMatchesNewline: boolean
}>

export type RegexSpan = Readonly<{
  start: number
  end: number
}>

export type RegexCapture = Readonly<{
  span: RegexSpan
  text: string
}>

export type RegexMatch = Readonly<{
  span: RegexSpan
  text: string
  captures: ReadonlyArray<Maybe<RegexCapture>>
  named: PersistentMap<string, Maybe<RegexCapture>>
}>

type PatternScalar = Readonly<{
  value: string
  point: number
  utf16: number
  byte: number
}>

type Property =
  | "alphabetic"
  | "whitespace"
  | "mark"
  | "decimal-number"
  | "connector-punctuation"
  | "letter"
  | "number"
  | "punctuation"
  | "symbol"
  | "separator"
  | "other"
  | Readonly<{ category: number }>

type ClassTerm =
  | Readonly<{ kind: "range"; start: number; end: number }>
  | Readonly<{ kind: "digit"; negated: boolean }>
  | Readonly<{ kind: "space"; negated: boolean }>
  | Readonly<{ kind: "word"; negated: boolean }>
  | Readonly<{ kind: "property"; property: Property; negated: boolean }>

type ScalarMatcher = Readonly<{
  terms: ReadonlyArray<ClassTerm>
  negated: boolean
  dot: boolean
}>

type Assertion = "start" | "end" | "absolute-start" | "absolute-end"

type Node =
  | Readonly<{ kind: "empty" }>
  | Readonly<{ kind: "consume"; matcher: ScalarMatcher }>
  | Readonly<{ kind: "assert"; assertion: Assertion }>
  | Readonly<{ kind: "concat"; parts: ReadonlyArray<Node> }>
  | Readonly<{ kind: "alternate"; branches: ReadonlyArray<Node> }>
  | Readonly<{
      kind: "capture"
      index: number
      name: string | undefined
      child: Node
    }>
  | Readonly<{
      kind: "repeat"
      child: Node
      minimum: number
      maximum: number | undefined
    }>

type Instruction =
  | { kind: "consume"; matcher: ScalarMatcher; next: number }
  | { kind: "jump"; next: number }
  | { kind: "split"; first: number; second: number }
  | { kind: "save"; slot: number; next: number }
  | { kind: "assert"; assertion: Assertion; next: number }
  | { kind: "counter-reset"; counter: number; next: number }
  | {
      kind: "counter-split"
      counter: number
      minimum: number
      maximum: number | undefined
      body: number
      exit: number
      captureSlots: ReadonlyArray<number>
    }
  | { kind: "counter-increment"; counter: number; split: number }
  | { kind: "accept" }

type Patch = Readonly<{
  instruction: number
  field: "next" | "second" | "exit"
}>

type Fragment = Readonly<{
  start: number
  exits: ReadonlyArray<Patch>
}>

type NamedCapture = Readonly<{ name: string; index: number }>

type CompiledPattern = Readonly<{
  instructions: ReadonlyArray<Instruction>
  start: number
  captureCount: number
  counterCount: number
  namedCaptures: ReadonlyArray<NamedCapture>
}>

const regexBrand = Symbol("Seseragi.Regex")

export type Regex = Readonly<{
  [regexBrand]: CompiledPattern
  options: RegexOptions
}>

export const UnexpectedRegexEnd: RegexCompileErrorKind = Object.freeze({
  tag: "UnexpectedRegexEnd",
})

export const UnexpectedRegexToken = (value: string): RegexCompileErrorKind =>
  Object.freeze({ tag: "UnexpectedRegexToken", value })

export const InvalidRegexEscape: RegexCompileErrorKind = Object.freeze({
  tag: "InvalidRegexEscape",
})

export const InvalidRegexRange: RegexCompileErrorKind = Object.freeze({
  tag: "InvalidRegexRange",
})

export const InvalidRegexQuantifier: RegexCompileErrorKind = Object.freeze({
  tag: "InvalidRegexQuantifier",
})

export const DuplicateCaptureName = (value: string): RegexCompileErrorKind =>
  Object.freeze({ tag: "DuplicateCaptureName", value })

export const UnsupportedRegexFeature = (value: string): RegexCompileErrorKind =>
  Object.freeze({ tag: "UnsupportedRegexFeature", value })

const DEFAULT_OPTIONS: RegexOptions = Object.freeze({
  caseInsensitive: false,
  multiline: false,
  dotMatchesNewline: false,
})

export const defaultOptions = (_unit: Unit): RegexOptions => DEFAULT_OPTIONS

class ParseFailure {
  constructor(readonly error: RegexCompileError) {}
}

function patternScalars(pattern: string): ReadonlyArray<PatternScalar> {
  const scalars: PatternScalar[] = []
  let utf16 = 0
  let byte = 0
  for (const value of pattern) {
    const point = value.codePointAt(0)!
    scalars.push(Object.freeze({ value, point, utf16, byte }))
    utf16 += value.length
    byte += utf8Width(point)
  }
  return Object.freeze(scalars)
}

class Parser {
  private readonly scalars: ReadonlyArray<PatternScalar>
  private readonly byteLength: number
  private readonly captureNames = new globalThis.Set<string>()
  private readonly namedCaptures: NamedCapture[] = []
  private cursor = 0
  private captures = 0

  constructor(pattern: string) {
    this.scalars = patternScalars(pattern)
    this.byteLength = this.scalars.reduce(
      (length, scalar) => length + utf8Width(scalar.point),
      0
    )
  }

  parse(): Readonly<{
    node: Node
    captureCount: number
    namedCaptures: ReadonlyArray<NamedCapture>
  }> {
    const node = this.alternation()
    if (!this.done()) this.fail(UnexpectedRegexToken(this.peek()!.value))
    return Object.freeze({
      node,
      captureCount: this.captures,
      namedCaptures: Object.freeze([...this.namedCaptures]),
    })
  }

  private alternation(): Node {
    const branches = [this.concatenation()]
    while (this.take("|")) branches.push(this.concatenation())
    return branches.length === 1
      ? branches[0]!
      : Object.freeze({ kind: "alternate", branches: Object.freeze(branches) })
  }

  private concatenation(): Node {
    const parts: Node[] = []
    while (
      !this.done() &&
      this.peek()!.value !== ")" &&
      this.peek()!.value !== "|"
    ) {
      parts.push(this.repetition())
    }
    if (parts.length === 0) return Object.freeze({ kind: "empty" })
    return parts.length === 1
      ? parts[0]!
      : Object.freeze({ kind: "concat", parts: Object.freeze(parts) })
  }

  private repetition(): Node {
    const atom = this.atom()
    const scalar = this.peek()
    if (
      scalar === undefined ||
      (scalar.value !== "*" &&
        scalar.value !== "+" &&
        scalar.value !== "?" &&
        scalar.value !== "{")
    ) {
      return atom.node
    }
    if (!atom.quantifiable) this.fail(InvalidRegexQuantifier, scalar.byte)
    const offset = scalar.byte
    let minimum: number
    let maximum: number | undefined
    if (this.take("*")) {
      minimum = 0
      maximum = undefined
    } else if (this.take("+")) {
      minimum = 1
      maximum = undefined
    } else if (this.take("?")) {
      minimum = 0
      maximum = 1
    } else {
      this.take("{")
      minimum = this.quantifierNumber(offset)
      if (this.take("}")) {
        maximum = minimum
      } else {
        if (!this.take(",")) this.fail(InvalidRegexQuantifier, offset)
        maximum =
          this.peek()?.value === "}" ? undefined : this.quantifierNumber(offset)
        if (!this.take("}")) this.fail(InvalidRegexQuantifier, offset)
        if (maximum !== undefined && maximum < minimum) {
          this.fail(InvalidRegexQuantifier, offset)
        }
      }
    }
    if (this.peek()?.value === "?") {
      this.fail(UnsupportedRegexFeature("lazy quantifier"))
    }
    if (this.peek()?.value === "+") {
      this.fail(UnsupportedRegexFeature("possessive quantifier"))
    }
    if (this.peek()?.value === "*" || this.peek()?.value === "{") {
      this.fail(InvalidRegexQuantifier)
    }
    return Object.freeze({
      kind: "repeat",
      child: atom.node,
      minimum,
      maximum,
    })
  }

  private quantifierNumber(offset: number): number {
    if (!isAsciiDigit(this.peek()?.point)) {
      this.fail(InvalidRegexQuantifier, offset)
    }
    let value = 0
    while (isAsciiDigit(this.peek()?.point)) {
      value = value * 10 + (this.advance()!.point - 0x30)
      if (!Number.isSafeInteger(value))
        this.fail(InvalidRegexQuantifier, offset)
    }
    return value
  }

  private atom(): Readonly<{ node: Node; quantifiable: boolean }> {
    const scalar = this.advance()
    if (scalar === undefined) this.fail(UnexpectedRegexEnd)
    switch (scalar.value) {
      case "(":
        return { node: this.group(), quantifiable: true }
      case "[":
        return { node: this.characterClass(), quantifiable: true }
      case ".":
        return {
          node: consumeMatcher({ terms: [], negated: false, dot: true }),
          quantifiable: true,
        }
      case "^":
        return {
          node: Object.freeze({ kind: "assert", assertion: "start" }),
          quantifiable: false,
        }
      case "$":
        return {
          node: Object.freeze({ kind: "assert", assertion: "end" }),
          quantifiable: false,
        }
      case "\\":
        return this.escape(false, scalar.byte)
      case ")":
      case "]":
        return this.fail(UnexpectedRegexToken(scalar.value), scalar.byte)
      case "*":
      case "+":
      case "?":
      case "{":
        return this.fail(InvalidRegexQuantifier, scalar.byte)
      default:
        return {
          node: literalNode(scalar.point),
          quantifiable: true,
        }
    }
  }

  private group(): Node {
    let capturing = true
    let name: string | undefined
    let nameOffset: number | undefined
    if (this.take("?")) {
      const questionOffset = this.previous()!.byte
      if (this.take(":")) {
        capturing = false
      } else if (this.take("<")) {
        if (this.peek()?.value === "=" || this.peek()?.value === "!") {
          this.fail(UnsupportedRegexFeature("look-around"), questionOffset)
        }
        const capture = this.captureName(questionOffset)
        name = capture.name
        nameOffset = capture.offset
      } else {
        const marker = this.peek()?.value
        const feature =
          marker === "=" || marker === "!"
            ? "look-around"
            : marker === ">"
              ? "atomic group"
              : marker === "("
                ? "conditional"
                : marker === "R" || isAsciiDigit(this.peek()?.point)
                  ? "recursion"
                  : "inline flag"
        this.fail(UnsupportedRegexFeature(feature), questionOffset)
      }
    }
    const index = capturing ? this.captures++ : -1
    if (name !== undefined) {
      if (this.captureNames.has(name)) {
        this.fail(DuplicateCaptureName(name), nameOffset)
      }
      this.captureNames.add(name)
      this.namedCaptures.push(Object.freeze({ name, index }))
    }
    const child = this.alternation()
    if (!this.take(")")) {
      if (this.done()) this.fail(UnexpectedRegexEnd)
      this.fail(UnexpectedRegexToken(this.peek()!.value))
    }
    return capturing
      ? Object.freeze({ kind: "capture", index, name, child })
      : child
  }

  private captureName(
    offset: number
  ): Readonly<{ name: string; offset: number }> {
    const start = this.cursor
    const nameOffset = this.peek()?.byte ?? this.byteLength
    while (!this.done() && this.peek()!.value !== ">") this.cursor += 1
    if (this.done()) this.fail(UnexpectedRegexEnd)
    const values = this.scalars.slice(start, this.cursor)
    this.cursor += 1
    if (values.length === 0 || !isCaptureNameStart(values[0]!.point)) {
      this.fail(InvalidRegexEscape, offset)
    }
    if (values.slice(1).some((value) => !isCaptureNameContinue(value.point))) {
      this.fail(InvalidRegexEscape, offset)
    }
    return Object.freeze({
      name: values.map((value) => value.value).join(""),
      offset: nameOffset,
    })
  }

  private characterClass(): Node {
    const negated = this.take("^")
    const terms: ClassTerm[] = []
    let first = true
    while (true) {
      if (this.done()) this.fail(UnexpectedRegexEnd)
      if (this.peek()!.value === "]") {
        if (first) this.fail(UnexpectedRegexToken("]"))
        this.cursor += 1
        break
      }
      const left = this.classAtom()
      first = false
      if (
        this.peek()?.value === "-" &&
        this.scalars[this.cursor + 1]?.value !== "]"
      ) {
        const offset = this.advance()!.byte
        const right = this.classAtom()
        if (left.point === undefined || right.point === undefined) {
          this.fail(InvalidRegexRange, offset)
        }
        if (left.point > right.point) this.fail(InvalidRegexRange, offset)
        terms.push(
          Object.freeze({ kind: "range", start: left.point, end: right.point })
        )
      } else {
        terms.push(left.term)
      }
    }
    return consumeMatcher({ terms, negated, dot: false })
  }

  private classAtom(): Readonly<{
    term: ClassTerm
    point: number | undefined
  }> {
    const scalar = this.advance()
    if (scalar === undefined) this.fail(UnexpectedRegexEnd)
    if (scalar.value === "\\") {
      const escaped = this.escape(true, scalar.byte)
      if (
        escaped.node.kind !== "consume" ||
        escaped.node.matcher.terms.length !== 1
      ) {
        this.fail(InvalidRegexEscape, scalar.byte)
      }
      const term = escaped.node.matcher.terms[0]!
      const point =
        term.kind === "range" && term.start === term.end
          ? term.start
          : undefined
      return Object.freeze({ term, point })
    }
    return Object.freeze({
      term: Object.freeze({
        kind: "range",
        start: scalar.point,
        end: scalar.point,
      }),
      point: scalar.point,
    })
  }

  private escape(
    inClass: boolean,
    offset: number
  ): Readonly<{ node: Node; quantifiable: boolean }> {
    const escaped = this.advance()
    if (escaped === undefined) this.fail(UnexpectedRegexEnd)
    const classTerm = (term: ClassTerm) => ({
      node: consumeMatcher({ terms: [term], negated: false, dot: false }),
      quantifiable: true,
    })
    switch (escaped.value) {
      case "d":
      case "D":
        return classTerm(
          Object.freeze({ kind: "digit", negated: escaped.value === "D" })
        )
      case "s":
      case "S":
        return classTerm(
          Object.freeze({ kind: "space", negated: escaped.value === "S" })
        )
      case "w":
      case "W":
        return classTerm(
          Object.freeze({ kind: "word", negated: escaped.value === "W" })
        )
      case "p":
      case "P": {
        if (!this.take("{")) this.fail(InvalidRegexEscape, offset)
        const propertyStart = this.cursor
        while (!this.done() && this.peek()!.value !== "}") this.cursor += 1
        if (this.done()) this.fail(UnexpectedRegexEnd)
        const name = this.scalars
          .slice(propertyStart, this.cursor)
          .map((scalar) => scalar.value)
          .join("")
        this.cursor += 1
        const property = unicodeProperty(name)
        if (property === undefined) this.fail(InvalidRegexEscape, offset)
        return classTerm(
          Object.freeze({
            kind: "property",
            property,
            negated: escaped.value === "P",
          })
        )
      }
      case "A":
      case "z":
        if (inClass) this.fail(InvalidRegexEscape, offset)
        return {
          node: Object.freeze({
            kind: "assert",
            assertion:
              escaped.value === "A" ? "absolute-start" : "absolute-end",
          }),
          quantifiable: false,
        }
      case "n":
        return literalEscape(0x0a)
      case "r":
        return literalEscape(0x0d)
      case "t":
        return literalEscape(0x09)
      case "f":
        return literalEscape(0x0c)
      case "v":
        return literalEscape(0x0b)
      case "0":
        if (isAsciiDigit(this.peek()?.point)) {
          this.fail(UnsupportedRegexFeature("backreference"), offset)
        }
        return literalEscape(0)
      case "u":
        return literalEscape(this.unicodeEscape(offset))
      case "b":
        if (inClass) return literalEscape(0x08)
        return this.fail(UnsupportedRegexFeature("word boundary"), offset)
      case "B":
        return this.fail(UnsupportedRegexFeature("word boundary"), offset)
      case "k":
        return this.fail(UnsupportedRegexFeature("backreference"), offset)
      default:
        if (isAsciiDigit(escaped.point)) {
          this.fail(UnsupportedRegexFeature("backreference"), offset)
        }
        if (isAsciiLetter(escaped.point)) this.fail(InvalidRegexEscape, offset)
        return literalEscape(escaped.point)
    }
  }

  private unicodeEscape(offset: number): number {
    if (!this.take("{")) this.fail(InvalidRegexEscape, offset)
    let value = 0
    let digits = 0
    while (!this.done() && this.peek()!.value !== "}") {
      const digit = hexValue(this.advance()!.point)
      if (digit === undefined || digits === 6) {
        this.fail(InvalidRegexEscape, offset)
      }
      value = value * 16 + digit
      digits += 1
    }
    if (!this.take("}") || digits === 0 || !isUnicodeScalar(value)) {
      this.fail(InvalidRegexEscape, offset)
    }
    return value
  }

  private fail(kind: RegexCompileErrorKind, offset = this.offset()): never {
    throw new ParseFailure(Object.freeze({ kind, offset }))
  }

  private done(): boolean {
    return this.cursor === this.scalars.length
  }

  private peek(): PatternScalar | undefined {
    return this.scalars[this.cursor]
  }

  private previous(): PatternScalar | undefined {
    return this.scalars[this.cursor - 1]
  }

  private advance(): PatternScalar | undefined {
    const scalar = this.peek()
    if (scalar !== undefined) this.cursor += 1
    return scalar
  }

  private take(value: string): boolean {
    if (this.peek()?.value !== value) return false
    this.cursor += 1
    return true
  }

  private offset(): number {
    return this.peek()?.byte ?? this.byteLength
  }
}

function literalEscape(point: number): Readonly<{
  node: Node
  quantifiable: boolean
}> {
  return { node: literalNode(point), quantifiable: true }
}

function literalNode(point: number): Node {
  return consumeMatcher({
    terms: [Object.freeze({ kind: "range", start: point, end: point })],
    negated: false,
    dot: false,
  })
}

function consumeMatcher(matcher: {
  terms: ReadonlyArray<ClassTerm>
  negated: boolean
  dot: boolean
}): Node {
  return Object.freeze({
    kind: "consume",
    matcher: Object.freeze({
      terms: Object.freeze([...matcher.terms]),
      negated: matcher.negated,
      dot: matcher.dot,
    }),
  })
}

function isAsciiDigit(point: number | undefined): boolean {
  return point !== undefined && point >= 0x30 && point <= 0x39
}

function isAsciiLetter(point: number): boolean {
  return (point >= 0x41 && point <= 0x5a) || (point >= 0x61 && point <= 0x7a)
}

function isUnicodeScalar(point: number): boolean {
  return (
    point >= 0 && point <= 0x10ffff && !(point >= 0xd800 && point <= 0xdfff)
  )
}

function hexValue(point: number): number | undefined {
  if (point >= 0x30 && point <= 0x39) return point - 0x30
  if (point >= 0x41 && point <= 0x46) return point - 0x41 + 10
  if (point >= 0x61 && point <= 0x66) return point - 0x61 + 10
  return undefined
}

function isMark(point: number): boolean {
  const category = categoryIndex(point)
  return category >= 5 && category <= 7
}

function isDecimalNumber(point: number): boolean {
  return categoryIndex(point) === 8
}

function isConnectorPunctuation(point: number): boolean {
  return categoryIndex(point) === 11
}

function isWord(point: number): boolean {
  return (
    alphabetic(point) ||
    isMark(point) ||
    isDecimalNumber(point) ||
    isConnectorPunctuation(point)
  )
}

function isCaptureNameStart(point: number): boolean {
  return point === 0x5f || alphabetic(point)
}

function isCaptureNameContinue(point: number): boolean {
  return isCaptureNameStart(point) || isMark(point) || isDecimalNumber(point)
}

const CATEGORY_ALIASES: Readonly<Record<string, number>> = Object.freeze({
  uppercaseletter: 0,
  lu: 0,
  lowercaseletter: 1,
  ll: 1,
  titlecaseletter: 2,
  lt: 2,
  modifierletter: 3,
  lm: 3,
  otherletter: 4,
  lo: 4,
  nonspacingmark: 5,
  mn: 5,
  spacingmark: 6,
  mc: 6,
  enclosingmark: 7,
  me: 7,
  decimalnumber: 8,
  nd: 8,
  letternumber: 9,
  nl: 9,
  othernumber: 10,
  no: 10,
  connectorpunctuation: 11,
  pc: 11,
  dashpunctuation: 12,
  pd: 12,
  openpunctuation: 13,
  ps: 13,
  closepunctuation: 14,
  pe: 14,
  initialpunctuation: 15,
  pi: 15,
  finalpunctuation: 16,
  pf: 16,
  otherpunctuation: 17,
  po: 17,
  mathsymbol: 18,
  sm: 18,
  currencysymbol: 19,
  sc: 19,
  modifiersymbol: 20,
  sk: 20,
  othersymbol: 21,
  so: 21,
  spaceseparator: 22,
  zs: 22,
  lineseparator: 23,
  zl: 23,
  paragraphseparator: 24,
  zp: 24,
  control: 25,
  cc: 25,
  format: 26,
  cf: 26,
  privateuse: 27,
  co: 27,
  unassigned: 28,
  cn: 28,
})

function normalizePropertyName(name: string): string {
  let normalized = ""
  for (const scalar of name) {
    if (scalar !== "_" && scalar !== "-" && scalar !== " ") {
      const point = scalar.codePointAt(0)!
      normalized +=
        point >= 0x41 && point <= 0x5a
          ? String.fromCodePoint(point + 0x20)
          : scalar
    }
  }
  return normalized
}

function unicodeProperty(name: string): Property | undefined {
  const normalized = normalizePropertyName(name)
  const category = CATEGORY_ALIASES[normalized]
  if (category !== undefined) return Object.freeze({ category })
  switch (normalized) {
    case "alphabetic":
      return "alphabetic"
    case "whitespace":
      return "whitespace"
    case "mark":
    case "m":
      return "mark"
    case "decimalnumber":
      return "decimal-number"
    case "connectorpunctuation":
      return "connector-punctuation"
    case "letter":
    case "l":
      return "letter"
    case "number":
    case "n":
      return "number"
    case "punctuation":
    case "p":
      return "punctuation"
    case "symbol":
    case "s":
      return "symbol"
    case "separator":
    case "z":
      return "separator"
    case "other":
    case "c":
      return "other"
    default:
      return undefined
  }
}

function propertyMatches(property: Property, point: number): boolean {
  if (typeof property === "object")
    return categoryIndex(point) === property.category
  const category = categoryIndex(point)
  switch (property) {
    case "alphabetic":
      return alphabetic(point)
    case "whitespace":
      return whitespace(point)
    case "mark":
      return category >= 5 && category <= 7
    case "decimal-number":
      return category === 8
    case "connector-punctuation":
      return category === 11
    case "letter":
      return category >= 0 && category <= 4
    case "number":
      return category >= 8 && category <= 10
    case "punctuation":
      return category >= 11 && category <= 17
    case "symbol":
      return category >= 18 && category <= 21
    case "separator":
      return category >= 22 && category <= 24
    case "other":
      return category >= 25 && category <= 28
  }
}

function rawTermMatches(term: ClassTerm, point: number): boolean {
  switch (term.kind) {
    case "range":
      return point >= term.start && point <= term.end
    case "digit":
      return point >= 0x30 && point <= 0x39
    case "space":
      return whitespace(point)
    case "word":
      return isWord(point)
    case "property":
      return propertyMatches(term.property, point)
  }
}

function termMatches(
  term: ClassTerm,
  equivalents: ReadonlyArray<number>
): boolean {
  const matched = equivalents.some((point) => rawTermMatches(term, point))
  return term.kind !== "range" && term.negated ? !matched : matched
}

function isLineTerminator(point: number): boolean {
  return (
    point === 0x0a || point === 0x0d || point === 0x2028 || point === 0x2029
  )
}

function matcherMatches(
  matcher: ScalarMatcher,
  point: number,
  options: RegexOptions
): boolean {
  if (matcher.dot) {
    return options.dotMatchesNewline || !isLineTerminator(point)
  }
  const equivalents = options.caseInsensitive
    ? simpleFoldEquivalents(point)
    : [point]
  const matched = matcher.terms.some((term) => termMatches(term, equivalents))
  return matcher.negated ? !matched : matched
}

function nodeCaptureSlots(node: Node): ReadonlyArray<number> {
  const slots: number[] = []
  const visit = (current: Node): void => {
    switch (current.kind) {
      case "capture":
        slots.push(current.index * 2, current.index * 2 + 1)
        visit(current.child)
        break
      case "concat":
        for (const part of current.parts) visit(part)
        break
      case "alternate":
        for (const branch of current.branches) visit(branch)
        break
      case "repeat":
        visit(current.child)
        break
      case "empty":
      case "consume":
      case "assert":
        break
    }
  }
  visit(node)
  return Object.freeze(slots)
}

class Compiler {
  private readonly instructions: Instruction[] = []
  private counters = 0

  compile(
    node: Node,
    captureCount: number,
    namedCaptures: ReadonlyArray<NamedCapture>
  ): CompiledPattern {
    const fragment = this.node(node)
    const accept = this.emit({ kind: "accept" })
    this.patch(fragment.exits, accept)
    return Object.freeze({
      instructions: Object.freeze(
        this.instructions.map((instruction) => Object.freeze(instruction))
      ),
      start: fragment.start,
      captureCount,
      counterCount: this.counters,
      namedCaptures,
    })
  }

  private node(node: Node): Fragment {
    switch (node.kind) {
      case "empty": {
        const start = this.emit({ kind: "jump", next: -1 })
        return fragment(start, [{ instruction: start, field: "next" }])
      }
      case "consume": {
        const start = this.emit({
          kind: "consume",
          matcher: node.matcher,
          next: -1,
        })
        return fragment(start, [{ instruction: start, field: "next" }])
      }
      case "assert": {
        const start = this.emit({
          kind: "assert",
          assertion: node.assertion,
          next: -1,
        })
        return fragment(start, [{ instruction: start, field: "next" }])
      }
      case "concat": {
        let result = this.node(node.parts[0]!)
        for (const part of node.parts.slice(1)) {
          const next = this.node(part)
          this.patch(result.exits, next.start)
          result = fragment(result.start, next.exits)
        }
        return result
      }
      case "alternate": {
        const first = this.node(node.branches[0]!)
        const rest =
          node.branches.length === 2
            ? this.node(node.branches[1]!)
            : this.node(
                Object.freeze({
                  kind: "alternate",
                  branches: node.branches.slice(1),
                })
              )
        const start = this.emit({
          kind: "split",
          first: first.start,
          second: rest.start,
        })
        return fragment(start, [...first.exits, ...rest.exits])
      }
      case "capture": {
        const open = this.emit({ kind: "save", slot: node.index * 2, next: -1 })
        const body = this.node(node.child)
        const close = this.emit({
          kind: "save",
          slot: node.index * 2 + 1,
          next: -1,
        })
        this.patch([{ instruction: open, field: "next" }], body.start)
        this.patch(body.exits, close)
        return fragment(open, [{ instruction: close, field: "next" }])
      }
      case "repeat": {
        const counter = this.counters++
        const reset = this.emit({ kind: "counter-reset", counter, next: -1 })
        const split = this.emit({
          kind: "counter-split",
          counter,
          minimum: node.minimum,
          maximum: node.maximum,
          body: -1,
          exit: -1,
          captureSlots: nodeCaptureSlots(node.child),
        })
        const body = this.node(node.child)
        const increment = this.emit({
          kind: "counter-increment",
          counter,
          split,
        })
        this.patch([{ instruction: reset, field: "next" }], split)
        const splitInstruction = this.instructions[split]
        if (splitInstruction?.kind !== "counter-split") {
          throw new Error("invalid regex counter compiler state")
        }
        splitInstruction.body = body.start
        this.patch(body.exits, increment)
        return fragment(reset, [{ instruction: split, field: "exit" }])
      }
    }
  }

  private emit(instruction: Instruction): number {
    this.instructions.push(instruction)
    return this.instructions.length - 1
  }

  private patch(exits: ReadonlyArray<Patch>, target: number): void {
    for (const exit of exits) {
      const instruction = this.instructions[
        exit.instruction
      ] as unknown as Record<string, number>
      instruction[exit.field] = target
    }
  }
}

function fragment(start: number, exits: ReadonlyArray<Patch>): Fragment {
  return Object.freeze({ start, exits: Object.freeze(exits) })
}

export function compile(pattern: string): Either<RegexCompileError, Regex> {
  return compileWith(DEFAULT_OPTIONS, pattern)
}

export function compileWith(
  options: RegexOptions,
  pattern: string
): Either<RegexCompileError, Regex> {
  try {
    const parsed = new Parser(pattern).parse()
    const compiled = new Compiler().compile(
      parsed.node,
      parsed.captureCount,
      parsed.namedCaptures
    )
    return Right(
      Object.freeze({
        [regexBrand]: compiled,
        options: Object.freeze({
          caseInsensitive: options.caseInsensitive,
          multiline: options.multiline,
          dotMatchesNewline: options.dotMatchesNewline,
        }),
      })
    )
  } catch (error) {
    if (error instanceof ParseFailure) return Left(error.error)
    throw error
  }
}

type InputScalar = Readonly<{
  value: string
  point: number
  utf16: number
  byte: number
}>

type Input = Readonly<{
  text: string
  scalars: ReadonlyArray<InputScalar>
  utf16Boundaries: ReadonlyArray<number>
  byteBoundaries: ReadonlyArray<number>
}>

type Thread = Readonly<{
  instruction: number
  start: number
  captures: ReadonlyArray<number>
  counts: ReadonlyArray<number>
  iterationStarts: ReadonlyArray<number>
  iterationCaptures: ReadonlyArray<ReadonlyArray<number> | undefined>
}>

type InternalMatch = Readonly<{
  start: number
  end: number
  captures: ReadonlyArray<number>
}>

function inputScalars(text: string): Input {
  const scalars: InputScalar[] = []
  const utf16Boundaries = [0]
  const byteBoundaries = [0]
  let utf16 = 0
  let byte = 0
  for (const value of text) {
    const point = value.codePointAt(0)!
    scalars.push(Object.freeze({ value, point, utf16, byte }))
    utf16 += value.length
    byte += utf8Width(point)
    utf16Boundaries.push(utf16)
    byteBoundaries.push(byte)
  }
  return Object.freeze({
    text,
    scalars: Object.freeze(scalars),
    utf16Boundaries: Object.freeze(utf16Boundaries),
    byteBoundaries: Object.freeze(byteBoundaries),
  })
}

function newThread(pattern: CompiledPattern, start: number): Thread {
  return Object.freeze({
    instruction: pattern.start,
    start,
    captures: Object.freeze(new Array(pattern.captureCount * 2).fill(-1)),
    counts: Object.freeze(new Array(pattern.counterCount).fill(-1)),
    iterationStarts: Object.freeze(new Array(pattern.counterCount).fill(-1)),
    iterationCaptures: Object.freeze(
      new Array<ReadonlyArray<number> | undefined>(pattern.counterCount).fill(
        undefined
      )
    ),
  })
}

function moveThread(thread: Thread, instruction: number): Thread {
  return Object.freeze({ ...thread, instruction })
}

function replaceAt(
  values: ReadonlyArray<number>,
  index: number,
  value: number
): ReadonlyArray<number> {
  const result = [...values]
  result[index] = value
  return Object.freeze(result)
}

function replaceCaptureSnapshot(
  values: ReadonlyArray<ReadonlyArray<number> | undefined>,
  index: number,
  value: ReadonlyArray<number> | undefined
): ReadonlyArray<ReadonlyArray<number> | undefined> {
  const result = [...values]
  result[index] = value
  return Object.freeze(result)
}

function clearCaptureSlots(
  captures: ReadonlyArray<number>,
  slots: ReadonlyArray<number>
): ReadonlyArray<number> {
  if (slots.length === 0) return captures
  const result = [...captures]
  for (const slot of slots) result[slot] = -1
  return Object.freeze(result)
}

function threadStateKey(thread: Thread): string {
  return `${thread.instruction}|${thread.counts.join(",")}|${thread.iterationStarts.join(",")}`
}

function assertionMatches(
  assertion: Assertion,
  position: number,
  input: Input,
  options: RegexOptions
): boolean {
  if (assertion === "absolute-start") return position === 0
  if (assertion === "absolute-end") return position === input.scalars.length
  if (!options.multiline) {
    return assertion === "start"
      ? position === 0
      : position === input.scalars.length
  }
  if (assertion === "start") {
    if (position === 0) return true
    const previous = input.scalars[position - 1]!.point
    const current = input.scalars[position]?.point
    return (
      isLineTerminator(previous) && !(previous === 0x0d && current === 0x0a)
    )
  }
  if (position === input.scalars.length) return true
  const current = input.scalars[position]!.point
  const previous = input.scalars[position - 1]?.point
  return isLineTerminator(current) && !(current === 0x0a && previous === 0x0d)
}

function epsilonClosure(
  pattern: CompiledPattern,
  options: RegexOptions,
  input: Input,
  position: number,
  seeds: ReadonlyArray<Thread>,
  previousMatch: InternalMatch | undefined
): Readonly<{
  consuming: ReadonlyArray<Thread>
  match: InternalMatch | undefined
}> {
  const stack = [...seeds].reverse()
  const seen = new globalThis.Set<string>()
  const consuming: Thread[] = []
  let match = previousMatch
  let cutoff: number | undefined
  while (stack.length > 0) {
    const thread = stack.pop()!
    if (match !== undefined && thread.start > match.start) continue
    if (cutoff !== undefined && thread.start >= cutoff) continue
    const key = threadStateKey(thread)
    if (seen.has(key)) continue
    seen.add(key)
    const instruction = pattern.instructions[thread.instruction]!
    switch (instruction.kind) {
      case "consume":
        consuming.push(thread)
        break
      case "jump":
        stack.push(moveThread(thread, instruction.next))
        break
      case "split":
        stack.push(moveThread(thread, instruction.second))
        stack.push(moveThread(thread, instruction.first))
        break
      case "save":
        stack.push(
          Object.freeze({
            ...thread,
            instruction: instruction.next,
            captures: replaceAt(thread.captures, instruction.slot, position),
          })
        )
        break
      case "assert":
        if (assertionMatches(instruction.assertion, position, input, options)) {
          stack.push(moveThread(thread, instruction.next))
        }
        break
      case "counter-reset":
        stack.push(
          Object.freeze({
            ...thread,
            instruction: instruction.next,
            counts: replaceAt(thread.counts, instruction.counter, 0),
            iterationStarts: replaceAt(
              thread.iterationStarts,
              instruction.counter,
              -1
            ),
            iterationCaptures: replaceCaptureSnapshot(
              thread.iterationCaptures,
              instruction.counter,
              undefined
            ),
          })
        )
        break
      case "counter-split": {
        const count = thread.counts[instruction.counter]!
        const body = (): Thread =>
          Object.freeze({
            ...thread,
            instruction: instruction.body,
            captures: clearCaptureSlots(
              thread.captures,
              instruction.captureSlots
            ),
            iterationStarts: replaceAt(
              thread.iterationStarts,
              instruction.counter,
              position
            ),
            iterationCaptures: replaceCaptureSnapshot(
              thread.iterationCaptures,
              instruction.counter,
              thread.captures
            ),
          })
        const exit = (): Thread =>
          Object.freeze({
            ...thread,
            instruction: instruction.exit,
            counts: replaceAt(thread.counts, instruction.counter, -1),
            iterationStarts: replaceAt(
              thread.iterationStarts,
              instruction.counter,
              -1
            ),
            iterationCaptures: replaceCaptureSnapshot(
              thread.iterationCaptures,
              instruction.counter,
              undefined
            ),
          })
        if (count < instruction.minimum) {
          stack.push(body())
        } else if (
          instruction.maximum !== undefined &&
          count >= instruction.maximum
        ) {
          stack.push(exit())
        } else {
          stack.push(exit())
          stack.push(body())
        }
        break
      }
      case "counter-increment": {
        const previousCount = thread.counts[instruction.counter]!
        const rawCount = previousCount + 1
        const split = pattern.instructions[instruction.split]
        if (split?.kind !== "counter-split") {
          throw new Error("invalid regex counter runtime state")
        }
        const count = Math.min(rawCount, split.maximum ?? split.minimum)
        const didConsume =
          thread.iterationStarts[instruction.counter] !== position
        if (!didConsume) {
          const captures =
            previousCount >= split.minimum
              ? (thread.iterationCaptures[instruction.counter] ??
                thread.captures)
              : thread.captures
          stack.push(
            Object.freeze({
              ...thread,
              instruction: split.exit,
              captures,
              counts: replaceAt(thread.counts, instruction.counter, -1),
              iterationStarts: replaceAt(
                thread.iterationStarts,
                instruction.counter,
                -1
              ),
              iterationCaptures: replaceCaptureSnapshot(
                thread.iterationCaptures,
                instruction.counter,
                undefined
              ),
            })
          )
        } else {
          stack.push(
            Object.freeze({
              ...thread,
              instruction: instruction.split,
              counts: replaceAt(thread.counts, instruction.counter, count),
            })
          )
        }
        break
      }
      case "accept":
        match = Object.freeze({
          start: thread.start,
          end: position,
          captures: thread.captures,
        })
        cutoff = thread.start
        break
    }
  }
  return Object.freeze({
    consuming: Object.freeze(consuming),
    match,
  })
}

function findInternal(
  regex: Regex,
  input: Input,
  firstStart: number
): InternalMatch | undefined {
  const pattern = regex[regexBrand]
  let seeds: Thread[] = []
  let match: InternalMatch | undefined
  for (
    let position = firstStart;
    position <= input.scalars.length;
    position += 1
  ) {
    if (match === undefined) seeds.push(newThread(pattern, position))
    const closure = epsilonClosure(
      pattern,
      regex.options,
      input,
      position,
      seeds,
      match
    )
    match = closure.match
    if (position === input.scalars.length) return match
    seeds = []
    const scalar = input.scalars[position]!
    for (const thread of closure.consuming) {
      if (match !== undefined && thread.start > match.start) continue
      const instruction = pattern.instructions[thread.instruction]
      if (
        instruction?.kind === "consume" &&
        matcherMatches(instruction.matcher, scalar.point, regex.options)
      ) {
        seeds.push(moveThread(thread, instruction.next))
      }
    }
    if (seeds.length === 0 && match !== undefined) return match
  }
  return match
}

function captureValue(input: Input, start: number, end: number): RegexCapture {
  const span = Object.freeze({
    start: input.byteBoundaries[start]!,
    end: input.byteBoundaries[end]!,
  })
  return Object.freeze({
    span,
    text: copySubstring(
      input.text,
      input.utf16Boundaries[start]!,
      input.utf16Boundaries[end]!
    ),
  })
}

function publicMatch(
  regex: Regex,
  input: Input,
  matched: InternalMatch
): RegexMatch {
  const whole = captureValue(input, matched.start, matched.end)
  const captures: Maybe<RegexCapture>[] = []
  for (let index = 0; index < regex[regexBrand].captureCount; index += 1) {
    const start = matched.captures[index * 2]!
    const end = matched.captures[index * 2 + 1]!
    captures.push(
      start < 0 || end < 0 ? Nothing : Just(captureValue(input, start, end))
    )
  }
  let named = emptyMap<string, Maybe<RegexCapture>>()
  for (const capture of regex[regexBrand].namedCaptures) {
    named = insertMap(
      stringEq,
      stringHash,
      capture.name,
      captures[capture.index]!,
      named
    )
  }
  return Object.freeze({
    span: whole.span,
    text: whole.text,
    captures: Object.freeze(captures),
    named,
  })
}

function allMatches(regex: Regex, input: Input): ReadonlyArray<RegexMatch> {
  const matches: RegexMatch[] = []
  let next = 0
  while (next <= input.scalars.length) {
    const internal = findInternal(regex, input, next)
    if (internal === undefined) break
    matches.push(publicMatch(regex, input, internal))
    if (internal.end > internal.start) next = internal.end
    else if (internal.end < input.scalars.length) next = internal.end + 1
    else break
  }
  return Object.freeze(matches)
}

export function isMatch(regex: Regex, text: string): boolean {
  return findInternal(regex, inputScalars(text), 0) !== undefined
}

export function find(regex: Regex, text: string): Maybe<RegexMatch> {
  const input = inputScalars(text)
  const matched = findInternal(regex, input, 0)
  return matched === undefined
    ? Nothing
    : Just(publicMatch(regex, input, matched))
}

export function findAll(regex: Regex, text: string): ReadonlyArray<RegexMatch> {
  const input = inputScalars(text)
  return allMatches(regex, input)
}

export function split(regex: Regex, text: string): ReadonlyArray<string> {
  const input = inputScalars(text)
  const parts: string[] = []
  let previousByte = 0
  for (const matched of allMatches(regex, input)) {
    parts.push(copyByteSubstring(input, previousByte, matched.span.start))
    previousByte = matched.span.end
  }
  parts.push(
    copyByteSubstring(input, previousByte, input.byteBoundaries.at(-1)!)
  )
  return Object.freeze(parts)
}

function copyByteSubstring(input: Input, start: number, end: number): string {
  const startScalar = boundaryIndex(input.byteBoundaries, start)
  const endScalar = boundaryIndex(input.byteBoundaries, end)
  return copySubstring(
    input.text,
    input.utf16Boundaries[startScalar]!,
    input.utf16Boundaries[endScalar]!
  )
}

function boundaryIndex(
  boundaries: ReadonlyArray<number>,
  byte: number
): number {
  let low = 0
  let high = boundaries.length - 1
  while (low <= high) {
    const middle = Math.floor((low + high) / 2)
    const value = boundaries[middle]!
    if (value < byte) low = middle + 1
    else if (value > byte) high = middle - 1
    else return middle
  }
  throw new Error("regex produced a non-scalar UTF-8 boundary")
}

export function replaceAll(
  regex: Regex,
  replacement: string,
  text: string
): string {
  return replaceAllWith(regex, (_match) => replacement, text)
}

export function replaceAllWith(
  regex: Regex,
  replacement: (matched: RegexMatch) => string,
  text: string
): string {
  const input = inputScalars(text)
  const chunks: string[] = []
  let previousByte = 0
  for (const matched of allMatches(regex, input)) {
    chunks.push(copyByteSubstring(input, previousByte, matched.span.start))
    chunks.push(replacement(matched))
    previousByte = matched.span.end
  }
  chunks.push(
    copyByteSubstring(input, previousByte, input.byteBoundaries.at(-1)!)
  )
  return chunks.join("")
}

const REGEX_META = new globalThis.Set([
  "\\",
  ".",
  "^",
  "$",
  "|",
  "?",
  "*",
  "+",
  "(",
  ")",
  "[",
  "]",
  "{",
  "}",
])

function escapeRegex(text: string): string {
  let result = ""
  for (const scalar of text) {
    if (REGEX_META.has(scalar)) result += `\\${scalar}`
    else if (scalar === "\n") result += "\\n"
    else if (scalar === "\r") result += "\\r"
    else if (scalar === "\t") result += "\\t"
    else if (scalar === "\0") result += "\\u{0}"
    else result += scalar
  }
  return result
}

export { escapeRegex as escape }

export const regexCompileErrorKindEq: Eq<RegexCompileErrorKind> = Object.freeze(
  {
    eq:
      (left) =>
      (right): boolean =>
        left.tag === right.tag &&
        (!("value" in left) ||
          ("value" in right && left.value === right.value)),
  }
)

export const regexCompileErrorEq: Eq<RegexCompileError> = Object.freeze({
  eq:
    (left) =>
    (right): boolean =>
      intEq.eq(left.offset)(right.offset) &&
      regexCompileErrorKindEq.eq(left.kind)(right.kind),
})

export const regexOptionsEq: Eq<RegexOptions> = Object.freeze({
  eq:
    (left) =>
    (right): boolean =>
      boolEq.eq(left.caseInsensitive)(right.caseInsensitive) &&
      boolEq.eq(left.multiline)(right.multiline) &&
      boolEq.eq(left.dotMatchesNewline)(right.dotMatchesNewline),
})

export const regexSpanEq: Eq<RegexSpan> = Object.freeze({
  eq:
    (left) =>
    (right): boolean =>
      intEq.eq(left.start)(right.start) && intEq.eq(left.end)(right.end),
})

export const regexSpanOrd: Ord<RegexSpan> & Eq<RegexSpan> = Object.freeze({
  ...regexSpanEq,
  compare: (left) => (right) =>
    left.start < right.start
      ? Less
      : left.start > right.start
        ? Greater
        : left.end < right.end
          ? Less
          : left.end > right.end
            ? Greater
            : Equal,
})

export const regexCaptureEq: Eq<RegexCapture> = Object.freeze({
  eq:
    (left) =>
    (right): boolean =>
      regexSpanEq.eq(left.span)(right.span) &&
      stringEq.eq(left.text)(right.text),
})

const maybeCaptureEq: Eq<Maybe<RegexCapture>> = Object.freeze({
  eq:
    (left) =>
    (right): boolean =>
      left.tag === "Nothing"
        ? right.tag === "Nothing"
        : right.tag === "Just" && regexCaptureEq.eq(left.value)(right.value),
})

export const regexMatchEq: Eq<RegexMatch> = Object.freeze({
  eq:
    (left) =>
    (right): boolean =>
      regexSpanEq.eq(left.span)(right.span) &&
      stringEq.eq(left.text)(right.text) &&
      arrayEq(maybeCaptureEq).eq(left.captures)(right.captures) &&
      mapEq(stringEq, stringHash, maybeCaptureEq).eq(left.named)(right.named),
})
