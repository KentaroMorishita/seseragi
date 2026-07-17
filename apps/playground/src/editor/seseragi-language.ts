import {
  StreamLanguage,
  type StreamParser,
  type StringStream,
} from "@codemirror/language"
import { tags } from "@lezer/highlight"

const KEYWORDS = new Set([
  "as",
  "deriving",
  "do",
  "effect",
  "else",
  "fails",
  "fn",
  "for",
  "foreign",
  "from",
  "if",
  "impl",
  "import",
  "instance",
  "let",
  "match",
  "pub",
  "then",
  "trait",
  "type",
  "when",
  "where",
  "with",
])

const BUILTIN_TYPES = new Set([
  "Array",
  "BigInt",
  "Bool",
  "Char",
  "Effect",
  "Either",
  "Float",
  "Int",
  "List",
  "Map",
  "Maybe",
  "Never",
  "Range",
  "Set",
  "String",
  "Unit",
])

const BOOLEAN_VALUES = new Set(["False", "True"])

export function classifyIdentifier(identifier: string): string | null {
  if (KEYWORDS.has(identifier)) return "keyword"
  if (BOOLEAN_VALUES.has(identifier)) return "bool"
  if (BUILTIN_TYPES.has(identifier)) return "builtin-type"
  if (/^[A-Z]/u.test(identifier)) return "type-name"
  return "variable"
}

type State = {
  inBlockComment: boolean
  // Each active template stores 0 while scanning text and the nested brace
  // depth while scanning one of its interpolation expressions. A stack keeps
  // nested templates inside `${...}` structurally highlighted as well.
  templateInterpolationDepths: number[]
}

function scanTemplateText(stream: StringStream, state: State) {
  let escaped = false

  while (!stream.eol()) {
    if (!escaped && stream.match("${", false)) break

    const character = stream.next()
    if (escaped) {
      escaped = false
      continue
    }
    if (character === "\\") {
      escaped = true
      continue
    }
    if (character === "`") {
      state.templateInterpolationDepths.pop()
      break
    }
  }

  return "string"
}

const parser: StreamParser<State> = {
  tokenTable: {
    keyword: tags.keyword,
    bool: tags.bool,
    "builtin-type": tags.standard(tags.typeName),
    "type-name": tags.typeName,
    variable: tags.variableName,
    number: tags.number,
    string: tags.string,
    comment: tags.comment,
    operator: tags.operatorKeyword,
    punctuation: tags.punctuation,
  },
  startState: () => ({
    inBlockComment: false,
    templateInterpolationDepths: [],
  }),
  copyState: (state) => ({
    inBlockComment: state.inBlockComment,
    templateInterpolationDepths: [...state.templateInterpolationDepths],
  }),
  token(stream, state) {
    if (state.inBlockComment) {
      if (stream.skipTo("*/")) {
        stream.match("*/")
        state.inBlockComment = false
      } else {
        stream.skipToEnd()
      }
      return "comment"
    }

    const templateDepth = state.templateInterpolationDepths.at(-1)
    if (templateDepth === 0) {
      if (stream.match("${")) {
        state.templateInterpolationDepths[
          state.templateInterpolationDepths.length - 1
        ] = 1
        return "punctuation"
      }
      return scanTemplateText(stream, state)
    }

    if (stream.eatSpace()) return null
    if (stream.match("//")) {
      stream.skipToEnd()
      return "comment"
    }
    if (stream.match("/*")) {
      state.inBlockComment = true
      return "comment"
    }
    // A backtick directly before `[` selects Seseragi's persistent List
    // literal/comprehension syntax. Consume only the sigil here so the
    // collection contents keep their normal number, name, and operator tags.
    if (stream.match(/^`(?=\[)/u)) return "punctuation"
    if (stream.match("`")) {
      state.templateInterpolationDepths.push(0)
      return scanTemplateText(stream, state)
    }
    if (stream.match(/^"(?:[^"\\]|\\.)*"?/u)) return "string"
    if (stream.match(/^'(?:[^'\\]|\\.)*'?/u)) return "string"
    const interpolationDepth = state.templateInterpolationDepths.at(-1)
    if (interpolationDepth !== undefined && interpolationDepth > 0) {
      const activeTemplate = state.templateInterpolationDepths.length - 1
      if (stream.match("{")) {
        state.templateInterpolationDepths[activeTemplate] =
          interpolationDepth + 1
        return "punctuation"
      }
      if (stream.match("}")) {
        state.templateInterpolationDepths[activeTemplate] =
          interpolationDepth - 1
        return "punctuation"
      }
    }
    if (
      stream.match(/^\d(?:[\d_]*\d)?(?:\.\d(?:[\d_]*\d)?)?(?:[eE][+-]?\d+)?/u)
    ) {
      return "number"
    }
    if (
      stream.match(
        /^(?:\|>|<\||<-|->|>>=|<\*>|<\$>|<@>|\?\?|==|!=|<=|>=|&&|\|\||\+\+|\*\*|::|:=|\.\.\.|\.\.=|\.\.|[+$%&*+\-./:<=>?@\\^|~!])/u
      )
    ) {
      return "operator"
    }
    const identifier = stream.match(/^[\p{L}_][\p{L}\p{N}_']*/u)
    if (Array.isArray(identifier)) {
      return classifyIdentifier(identifier[0] ?? "")
    }
    if (stream.match(/^[()[\]{},;]/u)) return "punctuation"

    stream.next()
    return null
  },
}

export const seseragiLanguage = StreamLanguage.define(parser)
