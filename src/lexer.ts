/**
 * Seseragi Lexer - Tokenizes source code into tokens
 */

export enum TokenType {
  // Literals
  INTEGER = "INTEGER",
  FLOAT = "FLOAT",
  STRING = "STRING",
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
  MATCH = "MATCH",
  CASE = "CASE",
  IF = "IF",
  THEN = "THEN",
  ELSE = "ELSE",
  PURE = "PURE",
  PERFORM = "PERFORM",

  // Built-in functions
  PRINT = "PRINT",
  PUT_STR_LN = "PUT_STR_LN",
  TO_STRING = "TO_STRING",
  HEAD = "HEAD",
  TAIL = "TAIL",

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
  EQUAL = "EQUAL", // ==
  NOT_EQUAL = "NOT_EQUAL", // !=
  LESS_THAN = "LESS_THAN", // <
  GREATER_THAN = "GREATER_THAN", // >
  LESS_EQUAL = "LESS_EQUAL", // <=
  GREATER_EQUAL = "GREATER_EQUAL", // >=
  AND = "AND", // &&
  OR = "OR", // ||
  NOT = "NOT", // !

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
    ["match", TokenType.MATCH],
    ["case", TokenType.CASE],
    ["if", TokenType.IF],
    ["then", TokenType.THEN],
    ["else", TokenType.ELSE],
    ["pure", TokenType.PURE],
    ["perform", TokenType.PERFORM],
    ["print", TokenType.PRINT],
    ["putStrLn", TokenType.PUT_STR_LN],
    ["toString", TokenType.TO_STRING],
    ["head", TokenType.HEAD],
    ["tail", TokenType.TAIL],
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
        return this.makeToken(TokenType.QUESTION, char, startLine, startColumn)
      case ".":
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
        return this.makeToken(TokenType.DOT, char, startLine, startColumn)
      case "+":
        return this.makeToken(TokenType.PLUS, char, startLine, startColumn)
      case "*":
        return this.makeToken(TokenType.MULTIPLY, char, startLine, startColumn)
      case "/":
        if (this.peek() === "/") {
          return this.comment(startLine, startColumn)
        }
        return this.makeToken(TokenType.DIVIDE, char, startLine, startColumn)
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
        return this.makeToken(
          TokenType.FUNCTION_APPLICATION,
          char,
          startLine,
          startColumn
        )
      case "\\":
        return this.makeToken(TokenType.LAMBDA, char, startLine, startColumn)
      case "`":
        return this.makeToken(TokenType.BACKTICK, char, startLine, startColumn)
      case "\n":
        this.line++
        this.column = 1
        return this.makeToken(TokenType.NEWLINE, char, startLine, startColumn)
    }

    // Multi-character tokens
    if (char === "-") {
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
        char,
        startLine,
        startColumn,
        hadWhitespace
      )
    }

    if (char === "=") {
      if (this.peek() === "=") {
        this.advance()
        return this.makeToken(TokenType.EQUAL, "==", startLine, startColumn)
      }
      return this.makeToken(TokenType.ASSIGN, char, startLine, startColumn)
    }

    if (char === "!") {
      if (this.peek() === "=") {
        this.advance()
        return this.makeToken(TokenType.NOT_EQUAL, "!=", startLine, startColumn)
      }
      return this.makeToken(TokenType.NOT, char, startLine, startColumn)
    }

    if (char === "<") {
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
        return this.makeToken(
          TokenType.LESS_EQUAL,
          "<=",
          startLine,
          startColumn
        )
      }
      return this.makeToken(TokenType.LESS_THAN, char, startLine, startColumn)
    }

    if (char === ">") {
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
      return this.makeToken(
        TokenType.GREATER_THAN,
        char,
        startLine,
        startColumn
      )
    }

    if (char === "&" && this.peek() === "&") {
      this.advance()
      return this.makeToken(TokenType.AND, "&&", startLine, startColumn)
    }

    if (char === "|") {
      if (this.peek() === "|") {
        this.advance()
        return this.makeToken(TokenType.OR, "||", startLine, startColumn)
      }
      return this.makeToken(TokenType.PIPE, char, startLine, startColumn)
    }

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
}

// Convenience function for lexing
export function lex(source: string): Token[] {
  const lexer = new Lexer(source)
  return lexer.tokenize()
}
