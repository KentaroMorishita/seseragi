/**
 * Seseragi Parser - Recursive Descent Parser
 */

import { Token, TokenType, Lexer } from "./lexer"
import * as AST from "./ast"

export class ParseError extends Error {
  token: Token

  constructor(message: string, token: Token) {
    super(`${message} at line ${token.line}, column ${token.column}`)
    this.token = token
  }
}

export class Parser {
  private tokens: Token[]
  private current: number = 0

  constructor(source: string) {
    const lexer = new Lexer(source)
    this.tokens = lexer.tokenize()
  }

  parse(): AST.Program {
    const statements: AST.Statement[] = []

    while (!this.isAtEnd()) {
      if (this.peek().type === TokenType.NEWLINE) {
        this.advance()
        continue
      }

      const stmt = this.statement()
      if (stmt) {
        statements.push(stmt)
      }
    }

    return new AST.Program(statements)
  }

  // =============================================================================
  // Statements
  // =============================================================================

  private statement(): AST.Statement | null {
    try {
      if (this.match(TokenType.FN)) {
        return this.functionDeclaration(false)
      }

      if (this.match(TokenType.EFFECTFUL)) {
        this.consume(TokenType.FN, "Expected 'fn' after 'effectful'")
        return this.functionDeclaration(true)
      }

      if (this.match(TokenType.TYPE)) {
        return this.typeDeclaration()
      }

      if (this.match(TokenType.IMPL)) {
        return this.implBlock()
      }

      if (this.match(TokenType.IMPORT)) {
        return this.importDeclaration()
      }

      if (this.match(TokenType.LET)) {
        return this.variableDeclaration()
      }

      if (this.match(TokenType.RETURN)) {
        return this.returnStatement()
      }

      // Skip whitespace and newlines
      if (
        this.peek().type === TokenType.NEWLINE ||
        this.peek().type === TokenType.WHITESPACE
      ) {
        this.advance()
        return null
      }

      throw new ParseError("Unexpected token", this.peek())
    } catch (error) {
      this.synchronize()
      throw error
    }
  }

  private functionDeclaration(isEffectful: boolean): AST.FunctionDeclaration {
    const name = this.consume(
      TokenType.IDENTIFIER,
      "Expected function name"
    ).value

    const parameters: AST.Parameter[] = []
    const returnType = this.parseFunctionSignature(parameters)

    this.consume(TokenType.ASSIGN, "Expected '=' after function signature")

    const body = this.expression()

    return new AST.FunctionDeclaration(
      name,
      parameters,
      returnType,
      body,
      isEffectful,
      this.previous().line,
      this.previous().column
    )
  }

  private parseFunctionSignature(parameters: AST.Parameter[]): AST.Type {
    // Parse parameters until we reach the final return type
    while (this.check(TokenType.IDENTIFIER)) {
      const paramNameToken = this.peek()

      // Look ahead to see if this is a parameter or return type
      // Scan ahead to find the pattern:
      // param :Type -> (parameter)
      // param :Type = (this is wrong, shouldn't happen)
      // Type = (return type)

      let isParameter = false
      let lookahead = this.current

      // Check if next token is colon (indicating parameter)
      if (
        lookahead + 1 < this.tokens.length &&
        this.tokens[lookahead + 1].type === TokenType.COLON
      ) {
        // This looks like "param :Type"
        lookahead += 2 // Skip param and :

        // Skip the type (could be complex)
        while (
          lookahead < this.tokens.length &&
          this.tokens[lookahead].type !== TokenType.ARROW &&
          this.tokens[lookahead].type !== TokenType.ASSIGN
        ) {
          lookahead++
        }

        // If we found an arrow, it's a parameter
        if (
          lookahead < this.tokens.length &&
          this.tokens[lookahead].type === TokenType.ARROW
        ) {
          isParameter = true
        }
      }

      if (isParameter) {
        // Parse as parameter
        const paramName = this.advance().value
        this.consume(TokenType.COLON, "Expected ':' after parameter name")
        const paramType = this.parseType()
        this.consume(TokenType.ARROW, "Expected '->' after parameter type")

        parameters.push(
          new AST.Parameter(
            paramName,
            paramType,
            paramNameToken.line,
            paramNameToken.column
          )
        )
      } else {
        // This must be the return type
        return this.parseType()
      }
    }

    throw new ParseError("Expected return type", this.peek())
  }

  private typeDeclaration(): AST.TypeDeclaration {
    const name = this.consume(TokenType.IDENTIFIER, "Expected type name").value

    this.consume(TokenType.LEFT_BRACE, "Expected '{' after type name")

    const fields: AST.TypeField[] = []

    while (!this.check(TokenType.RIGHT_BRACE) && !this.isAtEnd()) {
      if (this.match(TokenType.NEWLINE)) continue

      const fieldName = this.consume(
        TokenType.IDENTIFIER,
        "Expected field name"
      ).value
      this.consume(TokenType.COLON, "Expected ':' after field name")
      const fieldType = this.parseType()

      fields.push(
        new AST.TypeField(
          fieldName,
          fieldType,
          this.previous().line,
          this.previous().column
        )
      )
    }

    this.consume(TokenType.RIGHT_BRACE, "Expected '}' after type fields")

    return new AST.TypeDeclaration(
      name,
      fields,
      this.previous().line,
      this.previous().column
    )
  }

  private implBlock(): AST.ImplBlock {
    const typeName = this.consume(
      TokenType.IDENTIFIER,
      "Expected type name"
    ).value

    this.consume(TokenType.LEFT_BRACE, "Expected '{' after type name")

    const methods: AST.MethodDeclaration[] = []
    const operators: AST.OperatorDeclaration[] = []
    let monoid: AST.MonoidDeclaration | undefined

    while (!this.check(TokenType.RIGHT_BRACE) && !this.isAtEnd()) {
      if (this.match(TokenType.NEWLINE)) continue

      if (this.match(TokenType.FN)) {
        methods.push(this.methodDeclaration())
      } else if (this.match(TokenType.OPERATOR)) {
        operators.push(this.operatorDeclaration())
      } else if (this.match(TokenType.MONOID)) {
        monoid = this.monoidDeclaration()
      } else {
        throw new ParseError(
          "Expected method, operator, or monoid declaration",
          this.peek()
        )
      }
    }

    this.consume(TokenType.RIGHT_BRACE, "Expected '}' after impl block")

    return new AST.ImplBlock(
      typeName,
      methods,
      operators,
      monoid,
      this.previous().line,
      this.previous().column
    )
  }

  private methodDeclaration(): AST.MethodDeclaration {
    const name = this.consume(
      TokenType.IDENTIFIER,
      "Expected method name"
    ).value

    const parameters: AST.Parameter[] = []
    const returnType = this.parseFunctionSignature(parameters)

    this.consume(TokenType.ASSIGN, "Expected '=' after method signature")

    const body = this.expression()

    return new AST.MethodDeclaration(
      name,
      parameters,
      returnType,
      body,
      this.previous().line,
      this.previous().column
    )
  }

  private operatorDeclaration(): AST.OperatorDeclaration {
    const operator = this.advance().value // operator symbol

    this.consume(TokenType.LEFT_PAREN, "Expected '(' after operator")

    const parameters: AST.Parameter[] = []

    // Parse first parameter
    const firstParam = this.consume(
      TokenType.IDENTIFIER,
      "Expected parameter name"
    ).value
    this.consume(TokenType.COMMA, "Expected ',' after first parameter")
    const secondParam = this.consume(
      TokenType.IDENTIFIER,
      "Expected second parameter name"
    ).value
    this.consume(TokenType.COLON, "Expected ':' after parameter name")
    const paramType = this.parseType()

    this.consume(TokenType.RIGHT_PAREN, "Expected ')' after parameters")
    this.consume(TokenType.ARROW, "Expected '->' after parameters")

    const returnType = this.parseType()

    this.consume(TokenType.LEFT_BRACE, "Expected '{' before operator body")
    const body = this.expression()
    this.consume(TokenType.RIGHT_BRACE, "Expected '}' after operator body")

    parameters.push(
      new AST.Parameter(
        "self",
        paramType,
        this.previous().line,
        this.previous().column
      )
    )
    parameters.push(
      new AST.Parameter(
        secondParam,
        paramType,
        this.previous().line,
        this.previous().column
      )
    )

    return new AST.OperatorDeclaration(
      operator,
      parameters,
      returnType,
      body,
      this.previous().line,
      this.previous().column
    )
  }

  private monoidDeclaration(): AST.MonoidDeclaration {
    this.consume(TokenType.LEFT_BRACE, "Expected '{' after 'monoid'")

    // Parse identity
    this.consume(TokenType.IDENTIFIER, "Expected 'identity'") // TODO: make this a keyword
    const identity = this.expression()

    // Parse operator
    const operator = this.operatorDeclaration()

    this.consume(TokenType.RIGHT_BRACE, "Expected '}' after monoid")

    return new AST.MonoidDeclaration(
      identity,
      operator,
      this.previous().line,
      this.previous().column
    )
  }

  private importDeclaration(): AST.ImportDeclaration {
    const module = this.consume(
      TokenType.IDENTIFIER,
      "Expected module name"
    ).value

    this.consume(TokenType.COLON, "Expected ':' after module name")
    this.consume(TokenType.COLON, "Expected '::' after module name")
    this.consume(TokenType.LEFT_BRACE, "Expected '{' after '::'")

    const items: AST.ImportItem[] = []

    do {
      const name = this.consume(
        TokenType.IDENTIFIER,
        "Expected import item name"
      ).value
      let alias: string | undefined

      if (this.match(TokenType.AS)) {
        alias = this.consume(TokenType.IDENTIFIER, "Expected alias name").value
      }

      items.push(
        new AST.ImportItem(
          name,
          alias,
          this.previous().line,
          this.previous().column
        )
      )
    } while (this.match(TokenType.COMMA))

    this.consume(TokenType.RIGHT_BRACE, "Expected '}' after import items")

    return new AST.ImportDeclaration(
      module,
      items,
      this.previous().line,
      this.previous().column
    )
  }

  private variableDeclaration(): AST.VariableDeclaration {
    const name = this.consume(
      TokenType.IDENTIFIER,
      "Expected variable name"
    ).value

    let type: AST.Type | undefined
    if (this.match(TokenType.COLON)) {
      type = this.parseType()
    }

    this.consume(TokenType.ASSIGN, "Expected '=' after variable name")
    const initializer = this.expression()

    return new AST.VariableDeclaration(
      name,
      initializer,
      type,
      this.previous().line,
      this.previous().column
    )
  }

  private returnStatement(): AST.ReturnStatement {
    const expr = this.expression()

    return new AST.ReturnStatement(
      expr,
      this.previous().line,
      this.previous().column
    )
  }

  // =============================================================================
  // Types
  // =============================================================================

  private parseType(): AST.Type {
    const token = this.advance()

    if (token.type === TokenType.IDENTIFIER) {
      const name = token.value

      // Check for generic types List<T>, Maybe<T>, etc.
      if (this.match(TokenType.LESS_THAN)) {
        const typeArgs: AST.Type[] = []

        do {
          typeArgs.push(this.parseType())
        } while (this.match(TokenType.COMMA))

        this.consume(
          TokenType.GREATER_THAN,
          "Expected '>' after type arguments"
        )

        return new AST.GenericType(name, typeArgs, token.line, token.column)
      }

      return new AST.PrimitiveType(name, token.line, token.column)
    }

    if (this.check(TokenType.LEFT_PAREN)) {
      // Function type (Int -> Int -> Bool)
      this.consume(TokenType.LEFT_PAREN, "Expected '('")
      const paramType = this.parseType()
      this.consume(TokenType.ARROW, "Expected '->'")
      const returnType = this.parseType()
      this.consume(TokenType.RIGHT_PAREN, "Expected ')'")

      return new AST.FunctionType(
        paramType,
        returnType,
        token.line,
        token.column
      )
    }

    throw new ParseError("Expected type", token)
  }

  // =============================================================================
  // Expressions
  // =============================================================================

  private expression(): AST.Expression {
    return this.conditionalExpression()
  }

  private conditionalExpression(): AST.Expression {
    if (this.match(TokenType.IS)) {
      const condition = this.binaryExpression()
      this.consume(TokenType.THEN, "Expected 'then' after condition")
      const thenExpr = this.binaryExpression()
      this.consume(TokenType.ELSE, "Expected 'else' after then expression")
      const elseExpr = this.binaryExpression()

      return new AST.ConditionalExpression(
        condition,
        thenExpr,
        elseExpr,
        this.previous().line,
        this.previous().column
      )
    }

    if (this.match(TokenType.MATCH)) {
      return this.matchExpression()
    }

    return this.binaryExpression()
  }

  private matchExpression(): AST.MatchExpression {
    const expr = this.binaryExpression()

    this.consume(TokenType.LEFT_BRACE, "Expected '{' after match expression")

    const cases: AST.MatchCase[] = []

    while (!this.check(TokenType.RIGHT_BRACE) && !this.isAtEnd()) {
      if (this.match(TokenType.NEWLINE)) continue

      const pattern = this.pattern()
      this.consume(TokenType.ARROW, "Expected '->' after pattern")
      const caseExpr = this.expression()

      cases.push(
        new AST.MatchCase(
          pattern,
          caseExpr,
          this.previous().line,
          this.previous().column
        )
      )
    }

    this.consume(TokenType.RIGHT_BRACE, "Expected '}' after match cases")

    return new AST.MatchExpression(
      expr,
      cases,
      this.previous().line,
      this.previous().column
    )
  }

  private pattern(): AST.Pattern {
    if (this.check(TokenType.IDENTIFIER)) {
      const name = this.advance().value

      // Constructor pattern
      if (this.check(TokenType.IDENTIFIER)) {
        const patterns: AST.Pattern[] = []

        while (this.check(TokenType.IDENTIFIER)) {
          patterns.push(this.pattern())
        }

        return new AST.ConstructorPattern(
          name,
          patterns,
          this.previous().line,
          this.previous().column
        )
      }

      return new AST.IdentifierPattern(
        name,
        this.previous().line,
        this.previous().column
      )
    }

    if (
      this.check(TokenType.INTEGER) ||
      this.check(TokenType.FLOAT) ||
      this.check(TokenType.STRING) ||
      this.check(TokenType.BOOLEAN)
    ) {
      const token = this.advance()
      let value: string | number | boolean = token.value

      if (token.type === TokenType.INTEGER) {
        value = parseInt(token.value)
      } else if (token.type === TokenType.FLOAT) {
        value = parseFloat(token.value)
      } else if (token.type === TokenType.BOOLEAN) {
        value = token.value === "True"
      }

      return new AST.LiteralPattern(value, token.line, token.column)
    }

    throw new ParseError("Expected pattern", this.peek())
  }

  private binaryExpression(): AST.Expression {
    return this.pipelineExpression()
  }

  private pipelineExpression(): AST.Expression {
    let expr = this.bindExpression()

    while (this.match(TokenType.PIPE)) {
      const right = this.bindExpression()
      expr = new AST.Pipeline(
        expr,
        right,
        this.previous().line,
        this.previous().column
      )
    }

    return expr
  }

  private bindExpression(): AST.Expression {
    let expr = this.reversePipeExpression()

    while (this.match(TokenType.BIND)) {
      const right = this.reversePipeExpression()
      expr = new AST.MonadBind(
        expr,
        right,
        this.previous().line,
        this.previous().column
      )
    }

    return expr
  }

  private reversePipeExpression(): AST.Expression {
    let expr = this.comparisonExpression()

    while (this.match(TokenType.REVERSE_PIPE)) {
      const right = this.comparisonExpression()
      expr = new AST.ReversePipe(
        expr,
        right,
        this.previous().line,
        this.previous().column
      )
    }

    return expr
  }

  private comparisonExpression(): AST.Expression {
    let expr = this.termExpression()

    while (
      this.match(
        TokenType.EQUAL,
        TokenType.NOT_EQUAL,
        TokenType.GREATER_THAN,
        TokenType.GREATER_EQUAL,
        TokenType.LESS_THAN,
        TokenType.LESS_EQUAL
      )
    ) {
      const operator = this.previous().value
      const right = this.termExpression()
      expr = new AST.BinaryOperation(
        expr,
        operator,
        right,
        this.previous().line,
        this.previous().column
      )
    }

    return expr
  }

  private termExpression(): AST.Expression {
    let expr = this.factorExpression()

    while (this.match(TokenType.PLUS, TokenType.MINUS)) {
      const operator = this.previous().value
      const right = this.factorExpression()
      expr = new AST.BinaryOperation(
        expr,
        operator,
        right,
        this.previous().line,
        this.previous().column
      )
    }

    return expr
  }

  private factorExpression(): AST.Expression {
    let expr = this.unaryExpression()

    while (this.match(TokenType.MULTIPLY, TokenType.DIVIDE)) {
      const operator = this.previous().value
      const right = this.unaryExpression()
      expr = new AST.BinaryOperation(
        expr,
        operator,
        right,
        this.previous().line,
        this.previous().column
      )
    }

    return expr
  }

  private unaryExpression(): AST.Expression {
    if (this.match(TokenType.NOT, TokenType.MINUS)) {
      const operator = this.previous().value
      const expr = this.unaryExpression()
      return new AST.UnaryOperation(
        operator,
        expr,
        this.previous().line,
        this.previous().column
      )
    }

    return this.callExpression()
  }

  private callExpression(): AST.Expression {
    let expr = this.primaryExpression()

    while (true) {
      if (this.match(TokenType.LEFT_PAREN)) {
        const args: AST.Expression[] = []

        if (!this.check(TokenType.RIGHT_PAREN)) {
          do {
            args.push(this.expression())
          } while (this.match(TokenType.COMMA))
        }

        this.consume(TokenType.RIGHT_PAREN, "Expected ')' after arguments")
        expr = new AST.FunctionCall(
          expr,
          args,
          this.previous().line,
          this.previous().column
        )
      } else {
        break
      }
    }

    return expr
  }

  private primaryExpression(): AST.Expression {
    if (this.match(TokenType.INTEGER)) {
      const value = parseInt(this.previous().value)
      return new AST.Literal(
        value,
        "integer",
        this.previous().line,
        this.previous().column
      )
    }

    if (this.match(TokenType.FLOAT)) {
      const value = parseFloat(this.previous().value)
      return new AST.Literal(
        value,
        "float",
        this.previous().line,
        this.previous().column
      )
    }

    if (this.match(TokenType.STRING)) {
      const value = this.previous().value
      return new AST.Literal(
        value,
        "string",
        this.previous().line,
        this.previous().column
      )
    }

    if (this.match(TokenType.BOOLEAN)) {
      const value = this.previous().value === "True"
      return new AST.Literal(
        value,
        "boolean",
        this.previous().line,
        this.previous().column
      )
    }

    if (this.match(TokenType.IDENTIFIER)) {
      const name = this.previous().value
      return new AST.Identifier(
        name,
        this.previous().line,
        this.previous().column
      )
    }

    if (this.match(TokenType.LEFT_PAREN)) {
      const expr = this.expression()
      this.consume(TokenType.RIGHT_PAREN, "Expected ')' after expression")
      return expr
    }

    throw new ParseError("Expected expression", this.peek())
  }

  // =============================================================================
  // Utility Methods
  // =============================================================================

  private match(...types: TokenType[]): boolean {
    for (const type of types) {
      if (this.check(type)) {
        this.advance()
        return true
      }
    }
    return false
  }

  private check(type: TokenType): boolean {
    if (this.isAtEnd()) return false
    return this.peek().type === type
  }

  private advance(): Token {
    if (!this.isAtEnd()) this.current++
    return this.previous()
  }

  private isAtEnd(): boolean {
    return this.peek().type === TokenType.EOF
  }

  private peek(): Token {
    return this.tokens[this.current]
  }

  private previous(): Token {
    return this.tokens[this.current - 1]
  }

  private consume(type: TokenType, message: string): Token {
    if (this.check(type)) return this.advance()

    throw new ParseError(message, this.peek())
  }

  private synchronize(): void {
    this.advance()

    while (!this.isAtEnd()) {
      if (this.previous().type === TokenType.SEMICOLON) return

      switch (this.peek().type) {
        case TokenType.FN:
        case TokenType.TYPE:
        case TokenType.LET:
        case TokenType.IMPL:
        case TokenType.IMPORT:
          return
      }

      this.advance()
    }
  }
}
