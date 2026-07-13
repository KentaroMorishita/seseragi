import { StreamLanguage, type StreamParser } from "@codemirror/language"
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

type State = { inBlockComment: boolean }

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
  startState: () => ({ inBlockComment: false }),
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

    if (stream.eatSpace()) return null
    if (stream.match("//")) {
      stream.skipToEnd()
      return "comment"
    }
    if (stream.match("/*")) {
      state.inBlockComment = true
      return "comment"
    }
    if (stream.match(/^`(?:[^`\\]|\\.)*`?/u)) return "string"
    if (stream.match(/^"(?:[^"\\]|\\.)*"?/u)) return "string"
    if (stream.match(/^'(?:[^'\\]|\\.)*'?/u)) return "string"
    if (
      stream.match(/^\d(?:[\d_]*\d)?(?:\.\d(?:[\d_]*\d)?)?(?:[eE][+-]?\d+)?/u)
    ) {
      return "number"
    }
    if (
      stream.match(
        /^(?:\|>|<\||<-|->|>>=|<\*>|<\$>|<@>|\?\?|==|!=|<=|>=|&&|\|\||\+\+|\*\*|::|:=|\.\.|[+$%&*+\-./:<=>?@\\^|~!])/u
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
