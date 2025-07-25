/**
 * Seseragi Lexer - Tokenizes source code into tokens
 */

export enum TokenType {
  // Literals
  INTEGER = "INTEGER",
  FLOAT = "FLOAT",
  STRING = "STRING",
  TEMPLATE_STRING = "TEMPLATE_STRING",
  BOOLEAN = "BOOLEAN",

  // Identifiers
  IDENTIFIER = "IDENTIFIER",

  // Keywords
  FN = "FN",
  TYPE = "TYPE",
  STRUCT = "STRUCT",
  LET = "LET",
  IMPL = "IMPL",
  MONOID = "MONOID",
  OPERATOR = "OPERATOR",
  RETURN = "RETURN",
  EFFECTFUL = "EFFECTFUL",
  ELEVATE = "ELEVATE",
  IMPORT = "IMPORT",
  AS = "AS",
  FROM = "FROM",
  MATCH = "MATCH",
  CASE = "CASE",
  IF = "IF",
  THEN = "THEN",
  ELSE = "ELSE",
  PROMISE = "PROMISE",
  RESOLVE = "RESOLVE",
  REJECT = "REJECT",
  PURE = "PURE",
  PERFORM = "PERFORM",
  WHEN = "WHEN",
  IS = "IS",

  // Built-in functions
  PRINT = "PRINT",
  PUT_STR_LN = "PUT_STR_LN",
  TO_STRING = "TO_STRING",
  TO_INT = "TO_INT",
  TO_FLOAT = "TO_FLOAT",
  HEAD = "HEAD",
  TAIL = "TAIL",
  TYPEOF = "TYPEOF",
  TYPEOF_WITH_ALIASES = "TYPEOF_WITH_ALIASES",

  // Operators
  PIPE = "PIPE", // |
  REVERSE_PIPE = "REVERSE_PIPE", // ~
  BIND = "BIND", // >>=
  FOLD_MONOID = "FOLD_MONOID", // >>>
  FUNCTION_APPLICATION = "FUNCTION_APPLICATION", // $
  MAP = "MAP", // <$>
  APPLY = "APPLY", // <*>
  ARROW = "ARROW", // ->
  RANGE = "RANGE", // ..
  RANGE_INCLUSIVE = "RANGE_INCLUSIVE", // ..=
  SPREAD = "SPREAD", // ...
  GENERATOR = "GENERATOR", // <-
  PLUS = "PLUS", // +
  MINUS = "MINUS", // -
  MULTIPLY = "MULTIPLY", // *
  DIVIDE = "DIVIDE", // /
  MODULO = "MODULO", // %
  POWER = "POWER", // **
  EQUAL = "EQUAL", // ==
  NOT_EQUAL = "NOT_EQUAL", // !=
  LESS_THAN = "LESS_THAN", // <
  GREATER_THAN = "GREATER_THAN", // >
  LESS_EQUAL = "LESS_EQUAL", // <=
  GREATER_EQUAL = "GREATER_EQUAL", // >=
  AND = "AND", // &&
  OR = "OR", // ||
  AMPERSAND = "AMPERSAND", // &
  NOT = "NOT", // !
  NULLISH_COALESCING = "NULLISH_COALESCING", // ??
  HEAD_OP = "HEAD_OP", // ^
  TAIL_OP = "TAIL_OP", // >>

  // Punctuation
  COLON = "COLON", // :
  SEMICOLON = "SEMICOLON", // ;
  COMMA = "COMMA", // ,
  DOT = "DOT", // .
  ASSIGN = "ASSIGN", // =
  QUESTION = "QUESTION", // ?

  // Brackets
  LEFT_PAREN = "LEFT_PAREN", // (
  RIGHT_PAREN = "RIGHT_PAREN", // )
  LEFT_BRACE = "LEFT_BRACE", // {
  RIGHT_BRACE = "RIGHT_BRACE", // }
  LEFT_BRACKET = "LEFT_BRACKET", // [
  RIGHT_BRACKET = "RIGHT_BRACKET", // ]

  // Special
  BACKTICK = "BACKTICK", // `
  TEMPLATE_START = "TEMPLATE_START", // ${
  TEMPLATE_END = "TEMPLATE_END", // }
  LAMBDA = "LAMBDA", // \
  WILDCARD = "WILDCARD", // _
  NEWLINE = "NEWLINE",
  EOF = "EOF",
  WHITESPACE = "WHITESPACE",
  COMMENT = "COMMENT",
}

export interface Token {
  type: TokenType
  value: string
  line: number
  column: number
  hasLeadingWhitespace?: boolean
}

export class Lexer {
  private source: string
  private current: number = 0
  private line: number = 1
  private column: number = 1
  private templateDepth: number = 0 // テンプレート内の入れ子レベル

  private keywords: Map<string, TokenType> = new Map([
    ["fn", TokenType.FN],
    ["type", TokenType.TYPE],
    ["struct", TokenType.STRUCT],
    ["let", TokenType.LET],
    ["impl", TokenType.IMPL],
    ["monoid", TokenType.MONOID],
    ["operator", TokenType.OPERATOR],
    ["return", TokenType.RETURN],
    ["effectful", TokenType.EFFECTFUL],
    ["elevate", TokenType.ELEVATE],
    ["import", TokenType.IMPORT],
    ["as", TokenType.AS],
    ["from", TokenType.FROM],
    ["match", TokenType.MATCH],
    ["case", TokenType.CASE],
    ["if", TokenType.IF],
    ["then", TokenType.THEN],
    ["else", TokenType.ELSE],
    ["promise", TokenType.PROMISE],
    ["resolve", TokenType.RESOLVE],
    ["reject", TokenType.REJECT],
    ["when", TokenType.WHEN],
    ["is", TokenType.IS],
    ["pure", TokenType.PURE],
    ["perform", TokenType.PERFORM],
    ["print", TokenType.PRINT],
    ["putStrLn", TokenType.PUT_STR_LN],
    ["toString", TokenType.TO_STRING],
    ["toInt", TokenType.TO_INT],
    ["toFloat", TokenType.TO_FLOAT],
    ["head", TokenType.HEAD],
    ["tail", TokenType.TAIL],
    ["typeof", TokenType.TYPEOF],
    ["typeof'", TokenType.TYPEOF_WITH_ALIASES],
    ["True", TokenType.BOOLEAN],
    ["False", TokenType.BOOLEAN],
  ])

  constructor(source: string) {
    this.source = source
  }

  tokenize(): Token[] {
    const tokens: Token[] = []

    while (!this.isAtEnd()) {
      const token = this.nextToken()
      if (token) {
        tokens.push(token)
      }
    }

    tokens.push({
      type: TokenType.EOF,
      value: "",
      line: this.line,
      column: this.column,
    })

    return tokens
  }

  private handleSingleCharTokens(
    char: string,
    startLine: number,
    startColumn: number,
    _hadWhitespace: boolean
  ): Token | null {
    switch (char) {
      case "(":
        return this.makeToken(
          TokenType.LEFT_PAREN,
          char,
          startLine,
          startColumn
        )
      case ")":
        return this.makeToken(
          TokenType.RIGHT_PAREN,
          char,
          startLine,
          startColumn
        )
      case "{":
        return this.makeToken(
          TokenType.LEFT_BRACE,
          char,
          startLine,
          startColumn
        )
      case "}":
        if (this.templateDepth > 0) {
          this.templateDepth--
          return this.makeToken(
            TokenType.TEMPLATE_END,
            char,
            startLine,
            startColumn
          )
        }
        return this.makeToken(
          TokenType.RIGHT_BRACE,
          char,
          startLine,
          startColumn
        )
      case "[":
        return this.makeToken(
          TokenType.LEFT_BRACKET,
          char,
          startLine,
          startColumn
        )
      case "]":
        return this.makeToken(
          TokenType.RIGHT_BRACKET,
          char,
          startLine,
          startColumn
        )
      case ",":
        return this.makeToken(TokenType.COMMA, char, startLine, startColumn)
      case ";":
        return this.makeToken(TokenType.SEMICOLON, char, startLine, startColumn)
      case ":":
        return this.makeToken(TokenType.COLON, char, startLine, startColumn)
      case "?":
        return this.handleQuestionTokens(startLine, startColumn)
      case ".":
        return this.handleDotTokens(startLine, startColumn)
      case "+":
        return this.makeToken(TokenType.PLUS, char, startLine, startColumn)
      case "*":
        return this.handleStarTokens(startLine, startColumn)
      case "/":
        return this.handleSlashTokens(startLine, startColumn)
      case "%":
        return this.makeToken(TokenType.MODULO, char, startLine, startColumn)
      case "~":
        return this.makeToken(
          TokenType.REVERSE_PIPE,
          char,
          startLine,
          startColumn
        )
      case "$":
        return this.handleDollarTokens(startLine, startColumn)
      case "\\":
        return this.makeToken(TokenType.LAMBDA, char, startLine, startColumn)
      case "`":
        return this.handleBacktickTokens(startLine, startColumn)
      case "^":
        return this.makeToken(TokenType.HEAD_OP, char, startLine, startColumn)
      case "\n":
        this.line++
        this.column = 1
        return this.makeToken(TokenType.NEWLINE, char, startLine, startColumn)
      default:
        return null
    }
  }

  private handleDotTokens(startLine: number, startColumn: number): Token {
    if (this.peek() === "." && this.peekNext() === "=") {
      this.advance() // consume second .
      this.advance() // consume =
      return this.makeToken(
        TokenType.RANGE_INCLUSIVE,
        "..=",
        startLine,
        startColumn
      )
    }
    if (this.peek() === "." && this.peekNext() === ".") {
      this.advance() // consume second .
      this.advance() // consume third .
      return this.makeToken(TokenType.SPREAD, "...", startLine, startColumn)
    }
    if (this.peek() === ".") {
      this.advance() // consume second .
      return this.makeToken(TokenType.RANGE, "..", startLine, startColumn)
    }
    return this.makeToken(TokenType.DOT, ".", startLine, startColumn)
  }

  private handleQuestionTokens(startLine: number, startColumn: number): Token {
    if (this.peek() === "?") {
      this.advance() // consume second ?
      return this.makeToken(
        TokenType.NULLISH_COALESCING,
        "??",
        startLine,
        startColumn
      )
    }
    return this.makeToken(TokenType.QUESTION, "?", startLine, startColumn)
  }

  private handleStarTokens(startLine: number, startColumn: number): Token {
    if (this.peek() === "*") {
      this.advance()
      return this.makeToken(TokenType.POWER, "**", startLine, startColumn)
    }
    return this.makeToken(TokenType.MULTIPLY, "*", startLine, startColumn)
  }

  private handleSlashTokens(startLine: number, startColumn: number): Token {
    if (this.peek() === "/") {
      return this.comment(startLine, startColumn)
    }
    return this.makeToken(TokenType.DIVIDE, "/", startLine, startColumn)
  }

  private handleDollarTokens(startLine: number, startColumn: number): Token {
    if (this.peek() === "{") {
      this.advance() // consume {
      this.templateDepth++
      return this.makeToken(
        TokenType.TEMPLATE_START,
        "${",
        startLine,
        startColumn
      )
    }
    return this.makeToken(
      TokenType.FUNCTION_APPLICATION,
      "$",
      startLine,
      startColumn
    )
  }

  private handleBacktickTokens(startLine: number, startColumn: number): Token {
    // リスト糖衣構文 `[ かテンプレートリテラルかを判定
    if (this.peek() === "[") {
      return this.makeToken(TokenType.BACKTICK, "`", startLine, startColumn)
    } else {
      return this.templateString(startLine, startColumn)
    }
  }

  private handleMultiCharTokens(
    char: string,
    startLine: number,
    startColumn: number,
    hadWhitespace: boolean
  ): Token | null {
    switch (char) {
      case "-":
        return this.handleMinusTokens(startLine, startColumn, hadWhitespace)
      case "=":
        return this.handleEqualsTokens(startLine, startColumn)
      case "!":
        return this.handleExclamationTokens(startLine, startColumn)
      case "<":
        return this.handleLessTokens(startLine, startColumn)
      case ">":
        return this.handleGreaterTokens(startLine, startColumn)
      case "&":
        return this.handleAmpersandTokens(startLine, startColumn)
      case "|":
        return this.handlePipeTokens(startLine, startColumn)
      default:
        return null
    }
  }

  private handleMinusTokens(
    startLine: number,
    startColumn: number,
    hadWhitespace: boolean
  ): Token {
    if (this.peek() === ">") {
      this.advance()
      return this.makeToken(
        TokenType.ARROW,
        "->",
        startLine,
        startColumn,
        hadWhitespace
      )
    }
    // 負の数値リテラル: -123, -45.67
    if (this.isDigit(this.peek())) {
      return this.negativeNumber(startLine, startColumn)
    }
    return this.makeToken(
      TokenType.MINUS,
      "-",
      startLine,
      startColumn,
      hadWhitespace
    )
  }

  private handleEqualsTokens(startLine: number, startColumn: number): Token {
    if (this.peek() === "=") {
      this.advance()
      return this.makeToken(TokenType.EQUAL, "==", startLine, startColumn)
    }
    return this.makeToken(TokenType.ASSIGN, "=", startLine, startColumn)
  }

  private handleExclamationTokens(
    startLine: number,
    startColumn: number
  ): Token {
    if (this.peek() === "=") {
      this.advance()
      return this.makeToken(TokenType.NOT_EQUAL, "!=", startLine, startColumn)
    }
    return this.makeToken(TokenType.NOT, "!", startLine, startColumn)
  }

  private handleLessTokens(startLine: number, startColumn: number): Token {
    if (this.peek() === "$" && this.peekNext() === ">") {
      this.advance() // consume $
      this.advance() // consume >
      return this.makeToken(TokenType.MAP, "<$>", startLine, startColumn)
    }
    if (this.peek() === "*" && this.peekNext() === ">") {
      this.advance() // consume *
      this.advance() // consume >
      return this.makeToken(TokenType.APPLY, "<*>", startLine, startColumn)
    }
    if (this.peek() === "-") {
      this.advance() // consume -
      return this.makeToken(TokenType.GENERATOR, "<-", startLine, startColumn)
    }
    if (this.peek() === "=") {
      this.advance()
      return this.makeToken(TokenType.LESS_EQUAL, "<=", startLine, startColumn)
    }
    return this.makeToken(TokenType.LESS_THAN, "<", startLine, startColumn)
  }

  private handleGreaterTokens(startLine: number, startColumn: number): Token {
    if (this.peek() === "=") {
      this.advance()
      return this.makeToken(
        TokenType.GREATER_EQUAL,
        ">=",
        startLine,
        startColumn
      )
    }
    if (this.peek() === ">" && this.peekNext() === "=") {
      this.advance() // first >
      this.advance() // second >
      this.advance() // =
      return this.makeToken(TokenType.BIND, ">>=", startLine, startColumn)
    }
    if (this.peek() === ">" && this.peekNext() === ">") {
      this.advance() // first >
      this.advance() // second >
      this.advance() // third >
      return this.makeToken(
        TokenType.FOLD_MONOID,
        ">>>",
        startLine,
        startColumn
      )
    }
    if (this.peek() === ">") {
      this.advance() // second >
      return this.makeToken(TokenType.TAIL_OP, ">>", startLine, startColumn)
    }
    return this.makeToken(TokenType.GREATER_THAN, ">", startLine, startColumn)
  }

  private handleAmpersandTokens(
    startLine: number,
    startColumn: number
  ): Token | null {
    if (this.peek() === "&") {
      this.advance()
      return this.makeToken(TokenType.AND, "&&", startLine, startColumn)
    }
    return this.makeToken(TokenType.AMPERSAND, "&", startLine, startColumn)
  }

  private handlePipeTokens(startLine: number, startColumn: number): Token {
    if (this.peek() === "|") {
      this.advance()
      return this.makeToken(TokenType.OR, "||", startLine, startColumn)
    }
    return this.makeToken(TokenType.PIPE, "|", startLine, startColumn)
  }

  private handleLiteralsAndIdentifiers(
    char: string,
    startLine: number,
    startColumn: number
  ): Token {
    // String literals
    if (char === '"') {
      return this.string(startLine, startColumn)
    }

    // Number literals
    if (this.isDigit(char)) {
      return this.number(startLine, startColumn)
    }

    // Wildcard pattern
    if (char === "_") {
      return this.makeToken(TokenType.WILDCARD, char, startLine, startColumn)
    }

    // Identifiers and keywords
    if (this.isAlpha(char)) {
      return this.identifier(startLine, startColumn)
    }

    throw new Error(
      `Unexpected character: ${char} at line ${startLine}, column ${startColumn}`
    )
  }

  private nextToken(): Token | null {
    const hadWhitespace = this.hasWhitespace()
    this.skipWhitespace()

    if (this.isAtEnd()) {
      return null
    }

    const _start = this.current
    const startLine = this.line
    const startColumn = this.column

    const char = this.advance()

    // Single character tokens
    const singleCharToken = this.handleSingleCharTokens(
      char,
      startLine,
      startColumn,
      hadWhitespace
    )
    if (singleCharToken) {
      return singleCharToken
    }

    // Multi-character tokens
    const multiCharToken = this.handleMultiCharTokens(
      char,
      startLine,
      startColumn,
      hadWhitespace
    )
    if (multiCharToken) {
      return multiCharToken
    }

    // Literals and identifiers
    return this.handleLiteralsAndIdentifiers(char, startLine, startColumn)
  }

  private string(startLine: number, startColumn: number): Token {
    let value = ""

    while (this.peek() !== '"' && !this.isAtEnd()) {
      if (this.peek() === "\n") {
        this.line++
        this.column = 1
      }
      value += this.advance()
    }

    if (this.isAtEnd()) {
      throw new Error(
        `Unterminated string at line ${startLine}, column ${startColumn}`
      )
    }

    // Consume closing "
    this.advance()

    return this.makeToken(TokenType.STRING, value, startLine, startColumn)
  }

  private number(startLine: number, startColumn: number): Token {
    // Back up to include the first digit
    this.current--
    this.column--

    let value = ""
    let isFloat = false

    while (this.isDigit(this.peek())) {
      value += this.advance()
    }

    // Look for decimal point
    if (this.peek() === "." && this.isDigit(this.peekNext())) {
      isFloat = true
      value += this.advance() // consume '.'

      while (this.isDigit(this.peek())) {
        value += this.advance()
      }
    }

    return this.makeToken(
      isFloat ? TokenType.FLOAT : TokenType.INTEGER,
      value,
      startLine,
      startColumn
    )
  }

  private negativeNumber(startLine: number, startColumn: number): Token {
    let value = "-"
    let isFloat = false

    while (this.isDigit(this.peek())) {
      value += this.advance()
    }

    // Look for decimal point
    if (this.peek() === "." && this.isDigit(this.peekNext())) {
      isFloat = true
      value += this.advance() // consume '.'

      while (this.isDigit(this.peek())) {
        value += this.advance()
      }
    }

    return this.makeToken(
      isFloat ? TokenType.FLOAT : TokenType.INTEGER,
      value,
      startLine,
      startColumn
    )
  }

  private identifier(startLine: number, startColumn: number): Token {
    // Back up to include the first character
    this.current--
    this.column--

    let value = ""

    while (this.isAlphaNumeric(this.peek())) {
      value += this.advance()
    }

    // 末尾のアポストロフィを処理（Haskellスタイル）
    while (this.peek() === "'") {
      value += this.advance()
    }

    const tokenType = this.keywords.get(value) || TokenType.IDENTIFIER
    return this.makeToken(tokenType, value, startLine, startColumn)
  }

  private hasWhitespace(): boolean {
    const char = this.peek()
    return char === " " || char === "\r" || char === "\t"
  }

  private skipWhitespace(): void {
    while (true) {
      const char = this.peek()
      if (char === " " || char === "\r" || char === "\t") {
        this.advance()
      } else {
        break
      }
    }
  }

  private makeToken(
    type: TokenType,
    value: string,
    line: number,
    column: number,
    hasLeadingWhitespace?: boolean
  ): Token {
    return { type, value, line, column, hasLeadingWhitespace }
  }

  private isAtEnd(): boolean {
    return this.current >= this.source.length
  }

  private advance(): string {
    const char = this.source.charAt(this.current)
    this.current++
    this.column++
    return char
  }

  private peek(): string {
    if (this.isAtEnd()) return "\0"
    return this.source.charAt(this.current)
  }

  private peekNext(): string {
    if (this.current + 1 >= this.source.length) return "\0"
    return this.source.charAt(this.current + 1)
  }

  private peekThird(): string {
    if (this.current + 2 >= this.source.length) return "\0"
    return this.source.charAt(this.current + 2)
  }

  private isDigit(char: string): boolean {
    return char >= "0" && char <= "9"
  }

  private isAlpha(char: string): boolean {
    return (
      (char >= "a" && char <= "z") ||
      (char >= "A" && char <= "Z") ||
      char === "_"
    )
  }

  private isAlphaNumeric(char: string): boolean {
    return this.isAlpha(char) || this.isDigit(char)
  }

  private comment(startLine: number, startColumn: number): Token {
    // Skip the first /
    this.advance()

    let value = ""

    // Read until end of line or end of file
    while (this.peek() !== "\n" && !this.isAtEnd()) {
      value += this.advance()
    }

    return this.makeToken(TokenType.COMMENT, value, startLine, startColumn)
  }

  private templateString(startLine: number, startColumn: number): Token {
    let value = ""

    while (this.peek() !== "`" && !this.isAtEnd()) {
      if (this.peek() === "\n") {
        this.line++
        this.column = 1
      }
      value += this.advance()
    }

    if (this.isAtEnd()) {
      throw new Error(
        `Unterminated template string at line ${startLine}, column ${startColumn}`
      )
    }

    // Consume closing `
    this.advance()

    return this.makeToken(
      TokenType.TEMPLATE_STRING,
      value,
      startLine,
      startColumn
    )
  }
}

// Convenience function for lexing
export function lex(source: string): Token[] {
  const lexer = new Lexer(source)
  return lexer.tokenize()
}
