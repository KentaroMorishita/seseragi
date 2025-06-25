export enum TokenType {
  // Keywords
  FN = "FN",
  LET = "LET",
  TYPE = "TYPE",
  IMPL = "IMPL",
  MONOID = "MONOID",
  MATCH = "MATCH",
  EFFECTFUL = "EFFECTFUL",
  IS = "IS",
  THEN = "THEN",
  ELSE = "ELSE",

  // Types
  INT = "INT",
  FLOAT = "FLOAT",
  BOOL = "BOOL",
  STRING = "STRING",
  CHAR = "CHAR",
  UNIT = "UNIT",

  // Literals
  INT_LITERAL = "INT_LITERAL",
  FLOAT_LITERAL = "FLOAT_LITERAL",
  STRING_LITERAL = "STRING_LITERAL",
  CHAR_LITERAL = "CHAR_LITERAL",
  BOOL_LITERAL = "BOOL_LITERAL",

  // Identifiers
  IDENTIFIER = "IDENTIFIER",
  TYPE_IDENTIFIER = "TYPE_IDENTIFIER",

  // Operators
  PIPE = "PIPE",
  REVERSE_PIPE = "REVERSE_PIPE",
  BIND = "BIND",
  FOLD_MONOID = "FOLD_MONOID",
  ARROW = "ARROW",
  EQUALS = "EQUALS",
  PLUS = "PLUS",
  MINUS = "MINUS",
  MULTIPLY = "MULTIPLY",
  DIVIDE = "DIVIDE",
  MODULO = "MODULO",

  // Comparison
  EQ = "EQ",
  NE = "NE",
  LT = "LT",
  LE = "LE",
  GT = "GT",
  GE = "GE",

  // Delimiters
  LPAREN = "LPAREN",
  RPAREN = "RPAREN",
  LBRACE = "LBRACE",
  RBRACE = "RBRACE",
  LBRACKET = "LBRACKET",
  RBRACKET = "RBRACKET",
  COMMA = "COMMA",
  COLON = "COLON",
  DOUBLE_COLON = "DOUBLE_COLON",
  PIPE_DELIM = "PIPE_DELIM",

  // Whitespace and comments
  WHITESPACE = "WHITESPACE",
  NEWLINE = "NEWLINE",
  COMMENT = "COMMENT",

  // Special
  EOF = "EOF",
}

export interface Token {
  type: TokenType
  value: string
  line: number
  column: number
  start: number
  end: number
}

export class Lexer {
  private input: string
  private position: number = 0
  private line: number = 1
  private column: number = 1

  constructor(input: string) {
    this.input = input
  }

  tokenize(): Token[] {
    const tokens: Token[] = []

    while (this.position < this.input.length) {
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
      start: this.position,
      end: this.position,
    })

    return tokens
  }

  private nextToken(): Token | null {
    const start = this.position
    const startLine = this.line
    const startColumn = this.column

    const char = this.current()

    // Handle whitespace (preserve for formatting)
    if (this.isWhitespace(char)) {
      return this.readWhitespace(start, startLine, startColumn)
    }

    // Comments
    if (char === "/" && this.peek() === "/") {
      return this.readComment(start, startLine, startColumn)
    }

    // Newlines
    if (char === "\n") {
      this.advance()
      return {
        type: TokenType.NEWLINE,
        value: "\n",
        line: startLine,
        column: startColumn,
        start,
        end: this.position,
      }
    }

    // String literals
    if (char === '"') {
      return this.readString(start, startLine, startColumn)
    }

    // Character literals
    if (char === "'") {
      return this.readChar(start, startLine, startColumn)
    }

    // Numbers
    if (this.isDigit(char)) {
      return this.readNumber(start, startLine, startColumn)
    }

    // Multi-character operators
    if (char === ">" && this.peek() === ">" && this.peek(2) === "=") {
      this.advance()
      this.advance()
      this.advance()
      return {
        type: TokenType.BIND,
        value: ">>=",
        line: startLine,
        column: startColumn,
        start,
        end: this.position,
      }
    }

    if (char === ">" && this.peek() === ">" && this.peek(2) === ">") {
      this.advance()
      this.advance()
      this.advance()
      return {
        type: TokenType.FOLD_MONOID,
        value: ">>>",
        line: startLine,
        column: startColumn,
        start,
        end: this.position,
      }
    }

    if (char === "-" && this.peek() === ">") {
      this.advance()
      this.advance()
      return {
        type: TokenType.ARROW,
        value: "->",
        line: startLine,
        column: startColumn,
        start,
        end: this.position,
      }
    }

    if (char === ":" && this.peek() === ":") {
      this.advance()
      this.advance()
      return {
        type: TokenType.DOUBLE_COLON,
        value: "::",
        line: startLine,
        column: startColumn,
        start,
        end: this.position,
      }
    }

    if (char === "=" && this.peek() === "=") {
      this.advance()
      this.advance()
      return {
        type: TokenType.EQ,
        value: "==",
        line: startLine,
        column: startColumn,
        start,
        end: this.position,
      }
    }

    // Single character operators and delimiters
    const singleCharTokens: Record<string, TokenType> = {
      "|": TokenType.PIPE,
      "~": TokenType.REVERSE_PIPE,
      "=": TokenType.EQUALS,
      "+": TokenType.PLUS,
      "-": TokenType.MINUS,
      "*": TokenType.MULTIPLY,
      "/": TokenType.DIVIDE,
      "%": TokenType.MODULO,
      "(": TokenType.LPAREN,
      ")": TokenType.RPAREN,
      "{": TokenType.LBRACE,
      "}": TokenType.RBRACE,
      "[": TokenType.LBRACKET,
      "]": TokenType.RBRACKET,
      ",": TokenType.COMMA,
      ":": TokenType.COLON,
    }

    if (singleCharTokens[char]) {
      this.advance()
      return {
        type: singleCharTokens[char],
        value: char,
        line: startLine,
        column: startColumn,
        start,
        end: this.position,
      }
    }

    // Identifiers and keywords
    if (this.isAlpha(char) || char === "_") {
      return this.readIdentifier(start, startLine, startColumn)
    }

    // Unknown character
    this.advance()
    return {
      type: TokenType.IDENTIFIER,
      value: char,
      line: startLine,
      column: startColumn,
      start,
      end: this.position,
    }
  }

  private readWhitespace(start: number, line: number, column: number): Token {
    while (
      this.position < this.input.length &&
      this.isWhitespace(this.current()) &&
      this.current() !== "\n"
    ) {
      this.advance()
    }

    return {
      type: TokenType.WHITESPACE,
      value: this.input.slice(start, this.position),
      line,
      column,
      start,
      end: this.position,
    }
  }

  private readComment(start: number, line: number, column: number): Token {
    while (this.position < this.input.length && this.current() !== "\n") {
      this.advance()
    }

    return {
      type: TokenType.COMMENT,
      value: this.input.slice(start, this.position),
      line,
      column,
      start,
      end: this.position,
    }
  }

  private readString(start: number, line: number, column: number): Token {
    this.advance() // Skip opening quote

    while (this.position < this.input.length && this.current() !== '"') {
      if (this.current() === "\\") {
        this.advance() // Skip escape character
      }
      this.advance()
    }

    if (this.position < this.input.length) {
      this.advance() // Skip closing quote
    }

    return {
      type: TokenType.STRING_LITERAL,
      value: this.input.slice(start, this.position),
      line,
      column,
      start,
      end: this.position,
    }
  }

  private readChar(start: number, line: number, column: number): Token {
    this.advance() // Skip opening quote

    if (this.current() === "\\") {
      this.advance() // Skip escape character
    }
    this.advance() // Skip character

    if (this.position < this.input.length && this.current() === "'") {
      this.advance() // Skip closing quote
    }

    return {
      type: TokenType.CHAR_LITERAL,
      value: this.input.slice(start, this.position),
      line,
      column,
      start,
      end: this.position,
    }
  }

  private readNumber(start: number, line: number, column: number): Token {
    while (this.isDigit(this.current())) {
      this.advance()
    }

    let type = TokenType.INT_LITERAL

    // Check for float
    if (this.current() === "." && this.isDigit(this.peek())) {
      type = TokenType.FLOAT_LITERAL
      this.advance() // Skip dot
      while (this.isDigit(this.current())) {
        this.advance()
      }
    }

    return {
      type,
      value: this.input.slice(start, this.position),
      line,
      column,
      start,
      end: this.position,
    }
  }

  private readIdentifier(start: number, line: number, column: number): Token {
    while (this.isAlphaNumeric(this.current()) || this.current() === "_") {
      this.advance()
    }

    const value = this.input.slice(start, this.position)
    const type =
      this.getKeywordType(value) ||
      (this.isUpperCase(value[0])
        ? TokenType.TYPE_IDENTIFIER
        : TokenType.IDENTIFIER)

    return {
      type,
      value,
      line,
      column,
      start,
      end: this.position,
    }
  }

  private getKeywordType(value: string): TokenType | null {
    const keywords: Record<string, TokenType> = {
      fn: TokenType.FN,
      let: TokenType.LET,
      type: TokenType.TYPE,
      impl: TokenType.IMPL,
      monoid: TokenType.MONOID,
      match: TokenType.MATCH,
      effectful: TokenType.EFFECTFUL,
      is: TokenType.IS,
      then: TokenType.THEN,
      else: TokenType.ELSE,
      Int: TokenType.INT,
      Float: TokenType.FLOAT,
      Bool: TokenType.BOOL,
      String: TokenType.STRING,
      Char: TokenType.CHAR,
      Unit: TokenType.UNIT,
      True: TokenType.BOOL_LITERAL,
      False: TokenType.BOOL_LITERAL,
      Nothing: TokenType.IDENTIFIER,
      Just: TokenType.IDENTIFIER,
      Left: TokenType.IDENTIFIER,
      Right: TokenType.IDENTIFIER,
    }

    return keywords[value] || null
  }

  private current(): string {
    return this.position < this.input.length ? this.input[this.position] : ""
  }

  private peek(offset: number = 1): string {
    const pos = this.position + offset
    return pos < this.input.length ? this.input[pos] : ""
  }

  private advance(): void {
    if (this.position < this.input.length) {
      if (this.input[this.position] === "\n") {
        this.line++
        this.column = 1
      } else {
        this.column++
      }
      this.position++
    }
  }

  private isDigit(char: string): boolean {
    return char >= "0" && char <= "9"
  }

  private isAlpha(char: string): boolean {
    return (char >= "a" && char <= "z") || (char >= "A" && char <= "Z")
  }

  private isAlphaNumeric(char: string): boolean {
    return this.isAlpha(char) || this.isDigit(char)
  }

  private isWhitespace(char: string): boolean {
    return char === " " || char === "\t" || char === "\r"
  }

  private isUpperCase(char: string): boolean {
    return char >= "A" && char <= "Z"
  }
}
